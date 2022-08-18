use std::collections::HashSet;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use semver::{Version, VersionReq};

use crate::{
    index::Index,
    lock::Lock,
    manifest::{self, Manifest},
};

pub fn resolve(manifest: &Manifest, previous_lock: Option<&Lock>, index: &Index) -> Result<Lock> {
    log::trace!(
        "Resolving dependency tree{}",
        if previous_lock.is_some() {
            " with previous lock"
        } else {
            ""
        }
    );

    // Add the root dependencies.
    let mut candidates: Vec<Candidate> = manifest
        .dependencies
        .iter()
        .filter_map(|(name, dep)| {
            if let manifest::Dependency::Simple(req) = dep {
                Some(Candidate::new(name, req, index))
            } else {
                None
            }
        })
        .collect();

    // We should not remove any candidates before this index.
    let root_candidates_len = candidates.len();
    let mut i = 0;
    let mut failed = false;
    let mut complete = false;

    while !complete {
        while let Some(mut candidate) = candidates.get(i).and_then(|v| Some(v.clone())) {
            let mut next_version = None;
            // The available versions are sorted so that the top element
            // is always the latest version.
            while let Some(version) = candidate.available_versions.pop() {
                if !candidates
                    .iter()
                    .enumerate()
                    .filter(|&(idx, c)| c.version.is_some() && idx != i)
                    .any(|(_, i_candidate)| {
                        let i_version = i_candidate.version.as_ref().unwrap();
                        // Two versions are conflicting if they have the same major version,
                        // but a different minor versions. I.e. '1.2.x' is conflicting with '1.3.x'.
                        // Different major versions for the same dependency is allowed.
                        // This is for convenience when using libraries together,
                        // as different versions can't be mixed.
                        let is_conflicting = i_candidate.name == candidate.name
                            && i_version.major == version.major
                            && i_version != &version;

                        if is_conflicting {
                            log::trace!("Conflicting with: {} v{}", i_candidate.name, i_version);
                        }

                        is_conflicting
                    })
                {
                    // This version matches the requirements and does not
                    // come into conflict with any previously selected versions.
                    next_version = Some(version);
                    break;
                }
            }

            if let Some(next_version) = next_version {
                let deps = &index
                    .get(&candidate.name)
                    .unwrap()
                    .versions
                    .get(&next_version)
                    .unwrap()
                    .deps;

                candidates.extend(
                    deps.iter()
                        .map(|(dep, req)| Candidate::new(dep, req, index)),
                );

                // "Commit" this candidate to the list of candidates.
                // Will update the selected versions and the available versions left.
                candidate.version = Some(next_version.clone());
                candidates[i] = candidate;

                i += 1;
            } else {
                if i >= root_candidates_len {
                    // There were conflicts while selecting a version for this dependency
                    // so we invalidate the rest of the candidates by removing them.
                    // This is important as the dependency tree might change when we
                    // backtrack, leaving unused dependencies.
                    candidates.truncate(i);
                } else if i == 0 {
                    // There was no more versions left for the first
                    // dependency that satisfies the requirements.
                    failed = true;
                    break;
                } else {
                    // We can't remove a root dependency so we just
                    // unset the version and make sure it is ready for
                    // picking another version.
                    candidates[i].version = None;
                    candidates[i].update_available_versions(index);
                }

                i -= 1;
            }
        }

        // All combinations have been tried.
        // This manifest can't be resolved.
        if failed {
            break;
        }

        // When conflicts arise and we remove the invalidated candidates,
        // some of the previous dependencies might not get re-iterated so
        // that their dependencies are added as candidates again.
        // We go over the candidate list again to make sure all dependencies
        // are present and if not, we add the missing dependencies and restart
        // resolution loop.
        let mut missing_dependencies = Vec::new();
        for candidate in &candidates {
            let deps = &index
                .get(&candidate.name)
                .unwrap()
                .versions
                .get(candidate.version.as_ref().unwrap())
                .unwrap()
                .deps;

            for (dep, req) in deps {
                if !candidates
                    .iter()
                    .filter(|&c| &c.name == dep)
                    .any(|c| req.matches(c.version.as_ref().unwrap()))
                {
                    missing_dependencies.push((dep, req));
                }
            }
        }

        if missing_dependencies.is_empty() {
            // All dependencies are present so the resolution is ready,
            // so we stop the resolution loop.
            complete = true;
        } else {
            // We need to resolve the missing dependencies.
            // The resolution loop will continue automatically.
            log::trace!("{:?}", missing_dependencies);
            candidates.extend(
                missing_dependencies
                    .iter()
                    .map(|(dep, req)| Candidate::new(dep, req, index)),
            );
        }
    }

    if failed {
        log::trace!("Resolution failed");
        Err(anyhow!("Failed to resolve dependencies"))
    } else {
        let mut lock = Lock::new();
        lock.reserve(candidates.len());
        for candidate in candidates {
            lock.entry(candidate.name)
                .or_insert_with(|| HashSet::new())
                .insert(candidate.version.unwrap());
        }

        Ok(lock)
    }
}

#[derive(Clone)]
struct Candidate {
    name: String,
    req: VersionReq,
    version: Option<Version>,
    available_versions: Vec<Version>,
}

impl Candidate {
    pub fn new(name: &str, req: &VersionReq, index: &Index) -> Self {
        let mut candidate = Self {
            name: name.to_owned(),
            req: req.clone(),
            version: None,
            available_versions: Vec::new(),
        };
        candidate.update_available_versions(index);
        candidate
    }

    pub fn update_available_versions(&mut self, index: &Index) {
        self.available_versions.clear();
        if let Some(entry) = index.get(&self.name) {
            self.available_versions.extend(
                entry
                    .versions
                    .keys()
                    .filter(|v| self.req.matches(v))
                    .map(|v| v.clone())
                    .sorted_unstable_by(|v1, v2| v1.cmp(&v2)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use maplit::hashmap;
    use semver::{Version, VersionReq};

    use crate::{
        index::{Index, IndexEntry, VersionMetadata},
        manifest::{Manifest, Package},
    };

    #[test]
    fn basic() {
        let (manifest, index) = get_test_data();

        simplelog::TermLogger::init(
            log::LevelFilter::Debug,
            Default::default(),
            simplelog::TerminalMode::Stdout,
            simplelog::ColorChoice::Auto,
        )
        .unwrap();

        super::resolve(&manifest, None, &index).unwrap();
    }

    fn get_test_data() -> (Manifest, Index) {
        let index: Index = hashmap! {
            String::from("b") => IndexEntry {
                url: url::Url::parse("http://localhost").unwrap(),
                versions: hashmap! {
                    Version::from_str("1.0.0").unwrap() => VersionMetadata {
                        rev: String::new(),
                        deps: hashmap! {
                            String::from("d") => VersionReq::from_str("=1.0").unwrap(),
                        }
                    },
                }
            },
            String::from("c") => IndexEntry {
                url: url::Url::parse("http://localhost").unwrap(),
                versions: hashmap! {
                    Version::from_str("1.0.0").unwrap() => VersionMetadata {
                        rev: String::new(),
                        deps: hashmap! {}
                    },
                    Version::from_str("1.1.0").unwrap() => VersionMetadata {
                        rev: String::new(),
                        deps: hashmap! {
                            String::from("d") => VersionReq::from_str("1.1").unwrap(),
                        }
                    },
                }
            },
            String::from("d") => IndexEntry {
                url: url::Url::parse("http://localhost").unwrap(),
                versions: hashmap! {
                    Version::from_str("1.0.0").unwrap() => VersionMetadata {
                        rev: String::new(),
                        deps: hashmap! {}
                    },
                    Version::from_str("1.1.0").unwrap() => VersionMetadata {
                        rev: String::new(),
                        deps: hashmap! {}
                    },
                }
            },
        };

        let manifest = Manifest {
            package: Package {
                name: String::from("a"),
                version: Version::new(1, 0, 0),
            },
            dependencies: hashmap! {
                String::from("b") => crate::manifest::Dependency::Simple(VersionReq::from_str("1.0").unwrap()),
                String::from("c") => crate::manifest::Dependency::Simple(VersionReq::from_str("1.0").unwrap()),
                String::from("d") => crate::manifest::Dependency::Simple(VersionReq::from_str("1.0").unwrap()),
            },
        };

        (manifest, index)
    }
}
