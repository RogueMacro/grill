use std::collections::{HashMap, HashSet};

use anyhow::{bail, Context, Result};
use indexmap::IndexSet;
use itertools::Itertools;
use semver::{Version, VersionReq};

use crate::{
    index::{self, Index},
    lock::Lock,
    manifest::Manifest,
};

pub fn resolve(
    manifest: &Manifest,
    lock: Option<&Lock>,
) -> Result<HashMap<String, HashSet<Version>>> {
    let mut activated = HashMap::new();
    if let Some(lock) = lock {
        activated.extend(lock.iter().filter_map(|(pkg, versions)| {
            manifest.dependencies.get(pkg).and_then(|req| {
                if versions.iter().any(|version| req.matches(version)) {
                    Some((pkg.as_str(), versions.clone()))
                } else {
                    None
                }
            })
        }));
    }

    let index = index::parse(false, false)?;
    activate_deps(
        &manifest.dependencies,
        &mut activated,
        &index,
        &mut IndexSet::new(),
    )?;

    Ok(activated
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect())
}

fn activate<'a>(
    dep: &'a str,
    req: &VersionReq,
    activated: &mut HashMap<&'a str, HashSet<Version>>,
    index: &'a Index,
    conflicts: &mut IndexSet<(&'a str, Version)>,
) -> Result<Version> {
    let mut local_conflicts = IndexSet::new();

    'version_loop: for (version, metadata) in index
        .get(dep)
        .with_context(|| format!("Package not found: '{}'", dep))?
        .versions
        .iter()
        .filter(|(version, _)| {
            req.matches(&version) && !conflicts.contains(&(dep, (*version).clone()))
        })
        .sorted_unstable_by(|(v1, _), (v2, _)| v2.cmp(&v1))
    {
        if let Some(versions) = activated.get(dep) {
            for v in versions.iter() {
                if v.major == version.major && v.minor != version.minor {
                    local_conflicts.insert((dep, v.clone()));
                    continue 'version_loop;
                }
            }
        }

        activated
            .entry(dep)
            .or_insert(HashSet::new())
            .insert(version.clone());

        if activate_deps(&metadata.deps, activated, index, conflicts).is_err() {
            if let Some(activated_versions) = activated.get_mut(dep) {
                activated_versions.remove(&version);
            }
            continue 'version_loop;
        }

        return Ok(version.clone());
    }

    conflicts.extend(local_conflicts);

    bail!("No version satisfying requirement {} of {}", req, dep)
}

fn activate_deps<'a>(
    deps: &'a HashMap<String, VersionReq>,
    activated: &mut HashMap<&'a str, HashSet<Version>>,
    index: &'a Index,
    conflicts: &mut IndexSet<(&'a str, Version)>,
) -> Result<()> {
    let mut i = 0;
    let mut retry = true;
    let mut result = Ok(());
    'retry_loop: while i < 2 && retry {
        i += 1;
        retry = false;
        result = Ok(());
        for (dep, req) in deps.iter() {
            let conflicts_before = conflicts.len();
            if let Err(err) = activate(dep, req, activated, index, conflicts) {
                let conflicts_after = conflicts.len();
                if conflicts_after > conflicts_before {
                    retry = true;
                    let conflicting = conflicts.last().unwrap();
                    if let Some(activated_versions) = activated.get_mut(conflicting.0) {
                        activated_versions.remove(&conflicting.1);
                    }
                }

                result = Err(err);
                continue 'retry_loop;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use indexmap::IndexSet;
    use itertools::Itertools;
    use maplit::hashmap;
    use semver::{Version, VersionReq};

    use crate::{
        index::{Index, IndexEntry, VersionMetadata},
        manifest::{Manifest, Package},
    };

    #[test]
    fn basic() {
        for _ in 0..1000 {
            let index: Index = hashmap! {
                String::from("a") => IndexEntry {
                    url: url::Url::parse("http://localhost").unwrap(),
                    versions: hashmap! {
                        Version::from_str("0.1.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("=0.1.0").unwrap()
                            }
                        },
                        Version::from_str("0.1.1").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("=0.1.0").unwrap()
                            }
                        },
                        Version::from_str("0.2.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("0.2").unwrap()
                            }
                        }
                    }
                },
                String::from("b") => IndexEntry {
                    url: url::Url::parse("http://localhost").unwrap(),
                    versions: hashmap! {
                        Version::from_str("0.1.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("=0.1.1").unwrap(),
                                String::from("d") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                        Version::from_str("0.1.1").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("=0.1.1").unwrap(),
                                String::from("d") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                        Version::from_str("0.2.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("=0.1.1").unwrap(),
                                String::from("d") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                        Version::from_str("0.3.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("c") => VersionReq::from_str("0.2").unwrap(),
                                String::from("d") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                    }
                },
                String::from("c") => IndexEntry {
                    url: url::Url::parse("http://localhost").unwrap(),
                    versions: hashmap! {
                        Version::from_str("0.1.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("d") => VersionReq::from_str("0.1").unwrap(),
                                String::from("e") => VersionReq::from_str("0.1").unwrap(),
                            }
                        },
                        Version::from_str("0.1.1").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("d") => VersionReq::from_str("0.1").unwrap(),
                                String::from("e") => VersionReq::from_str("0.1").unwrap(),
                            }
                        },
                        Version::from_str("0.2.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("d") => VersionReq::from_str("0.1").unwrap(),
                                String::from("e") => VersionReq::from_str("0.1").unwrap(),
                            }
                        },
                        Version::from_str("0.3.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("d") => VersionReq::from_str("0.2").unwrap(),
                                String::from("e") => VersionReq::from_str("0.1").unwrap(),
                            }
                        },
                        Version::from_str("0.4.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("d") => VersionReq::from_str("0.2").unwrap(),
                                String::from("e") => VersionReq::from_str("0.1").unwrap(),
                            }
                        },
                    }
                },
                String::from("d") => IndexEntry {
                    url: url::Url::parse("http://localhost").unwrap(),
                    versions: hashmap! {
                        Version::from_str("0.1.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("e") => VersionReq::from_str("0.1").unwrap(),
                            }
                        },
                        Version::from_str("0.1.1").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("e") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                        Version::from_str("0.2.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("e") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                        Version::from_str("0.2.1").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("e") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                        Version::from_str("0.2.2").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {
                                String::from("e") => VersionReq::from_str("0.2").unwrap(),
                            }
                        },
                    }
                },
                String::from("e") => IndexEntry {
                    url: url::Url::parse("http://localhost").unwrap(),
                    versions: hashmap! {
                        Version::from_str("0.1.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {}
                        },
                        Version::from_str("0.1.1").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {}
                        },
                        Version::from_str("0.1.2").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {}
                        },
                        Version::from_str("0.2.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {}
                        },
                        Version::from_str("1.0.0").unwrap() => VersionMetadata {
                            rev: String::new(),
                            deps: hashmap! {}
                        },
                    }
                },
            };

            let manifest = Manifest {
                package: Package {
                    name: String::from("test"),
                    version: Version::new(0, 1, 0),
                },
                dependencies: hashmap! {
                    String::from("c") => VersionReq::from_str("0.1").unwrap(),
                    String::from("d") => VersionReq::from_str("0.1").unwrap(),
                    String::from("e") => VersionReq::from_str("1.0").unwrap(),
                },
            };

            let mut activated = HashMap::new();
            let result = super::activate_deps(
                &manifest.dependencies,
                &mut activated,
                &index,
                &mut IndexSet::new(),
            );

            assert!(result.is_ok(), "Activation failed!");

            println!("{:#?}", activated);

            // The latest compatible version is not always selected.
            // TODO: Fix
            // let expected = hashmap! {
            //     "c" => HashSet::from_iter([Version::new(0, 1, 1)]),
            //     "d" => HashSet::from_iter([Version::new(0, 1, 0)]),
            //     "e" => HashSet::from_iter([Version::new(1, 0, 0), Version::new(0, 1, 2)]),
            // };
            // assert!(
            //     activated.len() == expected.len()
            //         && activated.iter().all(|(k, v)| {
            //             if !expected.contains_key(k) {
            //                 false
            //             } else {
            //                 let expected_v = expected.get(k).unwrap();
            //                 expected_v.is_subset(v) && v.is_subset(expected_v)
            //             }
            //         })
            // );

            assert!(activated.values().all(|vset| vset
                .iter()
                .map(|v| Version::new(v.major, v.minor, 0))
                .all_unique()));
        }
    }
}
