use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use console::Emoji;
use either::{self, Either};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use semver::Version;

use crate::{
    beef, index, lock,
    manifest::{self, Manifest},
    prelude::*,
};

type Packages = HashMap<(String, Either<Version, String>), (PathBuf, PathBuf)>;

const COMPASS: Emoji = Emoji("🧭 ", "");
const LOOKING_GLASS: Emoji = Emoji("🔍 ", "");
const TRUCK: Emoji = Emoji("🚚 ", "");
const PACKAGE: Emoji = Emoji("📦 ", "");
const SPAGHETTI: Emoji = Emoji("🍝 ", "");

pub fn make<P>(ws_path: P, quiet: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let ws_path = ws_path.as_ref();
    let manifest = Manifest::from_pkg(ws_path)?;

    if !quiet {
        println!(
            "{:>12} {} v{}",
            console::style("Make").bright().cyan(),
            manifest.package.name,
            manifest.package.version
        );
    }

    let multi = crate::log::get_multi_progress();

    // Invisible progress bar to create empty line between logs and progress bars
    if !quiet {
        let p =
            multi.add(ProgressBar::new(0).with_style(ProgressStyle::default_bar().template(" ")?));
        p.finish();
    }

    let index = make_step(
        multi,
        1,
        4,
        "Updating",
        "Up to date",
        &COMPASS,
        quiet,
        |_, _| {
            index::update(false, false)?;
            index::parse(false, false)
        },
    )?;

    let lock = make_step(
        multi,
        2,
        4,
        "Resolving",
        "Resolution ready",
        &LOOKING_GLASS,
        quiet,
        |_, _| {
            if !lock::validate(ws_path)? {
                lock::generate(ws_path, true, true)
            } else {
                lock::read(ws_path.join(crate::paths::LOCK_FILENAME))
            }
        },
    )?;

    let pkgs = make_step(
        multi,
        3,
        4,
        "Fetching",
        "Packages on disk",
        &TRUCK,
        quiet,
        |multi, _| {
            let progress = multi.add(
                ProgressBar::new(1).with_style(
                    ProgressStyle::default_bar()
                        .template("{prefix:>12} [{bar:40}] {msg:.bright.grey}")?
                        .progress_chars("=> "),
                ),
            );
            if !quiet {
                progress
                    .set_length(lock.values().map(|versions| versions.len()).sum::<usize>() as u64);
                progress.set_prefix(format!(
                    "{} / {}",
                    0,
                    progress
                        .length()
                        .ok_or_else(|| anyhow!("No progress length"))?
                ));
            }

            let mut pkgs = HashMap::new();
            for (pkg, versions) in lock {
                for version in versions {
                    progress.set_message(format!("{} 0%", pkg));
                    let (relative_path, full_path, fetched) = crate::ops::install(
                        ws_path,
                        &pkg,
                        &version,
                        Some(&index),
                        |install_progress| {
                            progress.set_message(format!(
                                "{} {}%",
                                pkg,
                                (install_progress.indexed_objects() as f32
                                    / install_progress.total_objects() as f32
                                    * 100f32)
                                    .floor(),
                            ));
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        },
                    )?;

                    if fetched {
                        progress.set_message(format!("{} (Build)", pkg));
                        let spinner = multi.add(
                            ProgressBar::new_spinner()
                                .with_message(pkg.clone())
                                .with_style(
                                    ProgressStyle::default_spinner()
                                        .template("{prefix:>12} {msg} {spinner}")?
                                        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈✔"),
                                ),
                        );
                        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

                        crate::ops::rebuild(&full_path, Some(&spinner))?;
                        spinner.finish_and_clear();
                        multi.remove(&spinner);
                    }

                    if !quiet {
                        if fetched {
                            multi.suspend(|| {
                                println!(
                                    "{:>12} {} v{}",
                                    console::style("Fetched").bright().cyan(),
                                    pkg,
                                    version
                                );
                            });
                        }

                        progress.inc(1);
                        progress.set_prefix(format!(
                            "{} / {}",
                            progress.position(),
                            progress
                                .length()
                                .ok_or_else(|| anyhow!("No progress length"))?
                        ));
                    }

                    pkgs.insert(
                        (pkg.clone(), either::Left(version.clone())),
                        (relative_path, full_path),
                    );
                }
            }

            for (name, dep) in manifest.git_deps() {
                progress.set_message(format!("{} 0%", name));
                let (relative_path, full_path, rev) = crate::ops::install_git(
                    ws_path,
                    &dep.git,
                    Some(&dep.rev),
                    Some(name),
                    |install_progress| {
                        progress.set_message(format!(
                            "{} {}%",
                            name,
                            (install_progress.indexed_objects() as f32
                                / install_progress.total_objects() as f32
                                * 100f32)
                                .round(),
                        ))
                    },
                )?;

                pkgs.insert(
                    (name.clone(), either::Right(rev)),
                    (relative_path, full_path),
                );

                if !quiet {
                    progress.inc(1);
                    progress.set_prefix(format!(
                        "{} / {}",
                        progress.position(),
                        progress
                            .length()
                            .ok_or_else(|| anyhow!("No progress length"))?
                    ));
                }
            }

            progress.finish_and_clear();
            Ok(pkgs)
        },
    )?;

    make_step(
        multi,
        4,
        4,
        "Building",
        "Workspace done",
        &PACKAGE,
        quiet,
        |_, _| {
            let ws_file_path = ws_path.join("BeefSpace.toml");
            let proj_file_path = ws_path.join("BeefProj.toml");

            let mut ws = if ws_file_path.exists() {
                toml::from_str(&fs::read_to_string(&ws_file_path)?)?
            } else {
                beef::BeefSpace::default()
            };

            let mut proj = if proj_file_path.exists() {
                beef::BeefProj::from_file(&proj_file_path)?
            } else {
                beef::BeefProj::new(manifest.package.name.clone(), &proj_file_path)
            };
            proj.project.name = manifest.package.name.clone();
            proj.save()?;

            let mut ws_package_folder = HashSet::new();

            ws.workspace.startup_project = proj.project.name;
            ws.projects.clear();
            ws.projects.insert(
                String::from("corlib"),
                beef::ProjectEntry {
                    path: crate::paths::beeflib("corlib"),
                    ..Default::default()
                },
            );

            ws.locked.clear();
            ws.locked.insert(String::from("corlib"));

            let mut connects = HashMap::new();

            connect(
                &manifest.package.name,
                Some(&either::Left(manifest.package.version)),
                (Path::new("."), ws_path),
                false,
                &mut SharedConnectData {
                    pkgs: &pkgs,
                    ws: &mut ws,
                    ws_package_folder: &mut ws_package_folder,
                    connects: &mut connects,
                },
            )?;

            ws.projects.get_mut(&manifest.package.name).unwrap().path = ".".into();

            ws.workspace_folders
                .insert(String::from("Packages"), ws_package_folder);
            fs::write(&ws_file_path, toml::to_string(&ws)?)?;

            Ok(())
        },
    )?;

    if !quiet {
        println!("\n{:>13}{}Enjoy your spaghetti!", " ", SPAGHETTI);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn make_step<F, T>(
    multi: &MultiProgress,
    step: i32,
    steps: i32,
    msg: &'static str,
    finish: &'static str,
    emoji: &Emoji,
    quiet: bool,
    func: F,
) -> Result<T>
where
    F: FnOnce(&MultiProgress, &ProgressBar) -> Result<T>,
{
    log::trace!("Make: {}", msg);
    let spinner_style = ProgressStyle::default_spinner()
        .template("{msg} {spinner}")?
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈✔");
    let progress = multi.add(
        // step as usize,
        ProgressBar::new_spinner()
            .with_message(format!(
                "{:>12} {}{}",
                console::style(format!("[{}/{}]", step, steps)).dim(),
                emoji,
                msg
            ))
            .with_style(spinner_style),
    );
    if !quiet {
        progress.enable_steady_tick(Duration::from_millis(100));
    }

    let result = func(multi, &progress)?;

    if !quiet {
        progress.finish_with_message(format!(
            "{:>12} {}{}",
            console::style(format!("[{}/{}]", step, steps)).bold().dim(),
            emoji,
            finish
        ));
    }

    Ok(result)
}

/// Link packages to their dependencies by setting the right names and paths
/// in the project files. Recursively connects all dependencies.
fn connect(
    pkg_name: &str,
    pkg_version: Option<&Either<Version, String>>,
    pkg_path: (&Path, &Path),
    is_pkg: bool,
    shared: &mut SharedConnectData,
) -> Result<String> {
    if shared.connects.len() > 10 {
        panic!();
    }
    let full_pkg_path = pkg_path.1.canonicalize()?;
    if let Some(ident) = shared.connects.get(&full_pkg_path) {
        return Ok(ident.clone());
    }

    let manifest = Manifest::from_pkg(pkg_path.1)?;
    let mut proj = beef::BeefProj::from_file(&pkg_path.1.join("BeefProj.toml"))?;
    let is_binary = proj.project.target_type == "BeefConsoleApplication";
    proj.dependencies.clear();
    if manifest.package.corlib {
        proj.dependencies
            .insert(String::from("corlib"), String::from("*"));
    }

    let pkg_ident = if let Some(pkg_version) = pkg_version {
        match pkg_version {
            either::Left(v) => format!("{}-{}", pkg_name, v),
            either::Right(rev) => format!("{}-{}", pkg_name, rev),
        }
    } else {
        pkg_name.to_owned()
    };

    shared.connects.insert(
        full_pkg_path.clone(),
        if is_pkg {
            pkg_ident.clone()
        } else {
            pkg_name.to_owned()
        },
    );

    proj.project.processor_macros.clear();

    'dep_loop: for (name, dep) in manifest.dependencies.iter() {
        log::debug!("Dependency: {}", name);
        if let manifest::Dependency::Local(local) = dep {
            let dep_path = pkg_path.1.join(&local.path);
            let dep_manifest = Manifest::from_pkg(&dep_path)?;

            let full_dep_path = dep_path.canonicalize()?;

            if dep_manifest.features.optional.values().any(|f| {
                if let manifest::Feature::Project(p) = f {
                    dep_path.join(p).canonicalize().unwrap() == full_pkg_path
                } else {
                    false
                }
            }) {
                proj.dependencies.insert(
                    shared
                        .connects
                        .get(&dep_path.canonicalize()?)
                        .unwrap()
                        .clone(),
                    String::from("*"),
                );
                continue 'dep_loop;
            }

            let dep_ident = if full_pkg_path.starts_with(&full_dep_path) {
                // We are a subpackage to this dependency
                connect(
                    &format!("{}/{}", pkg_name, name),
                    None,
                    (&pkg_path.0.join(&local.path), &dep_path),
                    is_pkg,
                    shared,
                )?
            } else if full_dep_path.starts_with(&full_pkg_path) {
                // Dependency is a subpackage to us
                connect(
                    name,
                    None, //Some(&either::Left(dep_manifest.package.version)),
                    (&pkg_path.0.join(&local.path), &dep_path),
                    is_pkg && !is_binary, // If we are a binary application then local dependencies should not be considered packages
                    shared,
                )?
            } else {
                // Dependency is an external package outside our root package
                connect(
                    name,
                    None,
                    (&pkg_path.0.join(&local.path), &dep_path),
                    is_pkg,
                    shared,
                )?
            };

            proj.dependencies.insert(
                if !is_pkg || is_binary {
                    name.to_owned()
                } else {
                    dep_ident.clone()
                },
                String::from("*"),
            );

            let features: Box<dyn Iterator<Item = &String>> = if local.default_features {
                Box::new(
                    local
                        .features
                        .iter()
                        .chain(dep_manifest.features.default.iter()),
                )
            } else {
                Box::new(local.features.iter())
            };

            let mut dep_proj = beef::BeefProj::from_file(&dep_path.join("BeefProj.toml"))?;
            if !shared.connects.contains_key(&dep_path) {
                dep_proj.project.processor_macros.clear();
                shared
                    .connects
                    .insert(dep_path.canonicalize()?, dep_ident.clone());
            }

            for feature in features {
                if let Some(manifest::Feature::Project(p)) =
                    dep_manifest.features.optional.get(feature)
                {
                    if full_dep_path.join(p).canonicalize()? == full_pkg_path {
                        continue;
                    }
                }

                log::debug!("Enabling feature {} of {}", feature, name);
                let feature_idents = enable_feature((&local.path, &dep_path), feature, shared)?;

                proj.dependencies
                    .extend(feature_idents.into_iter().map(|i| (i, String::from("*"))));

                dep_proj
                    .project
                    .processor_macros
                    .insert(format!("FEATURE_{}", feature.to_uppercase()));
            }
            dep_proj.save()?;

            continue;
        }

        for ((pkg, version), (relative_path, full_path)) in shared.pkgs.iter() {
            if pkg == name {
                let mut features = None;
                let mut default_features = true;
                let add = match version {
                    either::Left(version) => {
                        if let manifest::Dependency::Simple(req) = dep {
                            req.matches(version)
                        } else if let manifest::Dependency::Advanced(dep) = dep {
                            features = Some(&dep.features);
                            default_features = dep.default_features;
                            dep.req.matches(version)
                        } else {
                            false
                        }
                    }
                    either::Right(rev) => {
                        if let manifest::Dependency::Git(dep) = dep {
                            &dep.rev == rev
                        } else {
                            false
                        }
                    }
                };

                if add {
                    let mut dep_proj = beef::BeefProj::from_file(&full_path.join("BeefProj.toml"))?;

                    let ident =
                        connect(pkg, Some(version), (relative_path, full_path), true, shared)?;

                    proj.dependencies.insert(ident.clone(), String::from("*"));

                    let dep_manifest = Manifest::from_pkg(full_path)?;
                    if dep_manifest.features.optional.values().any(|f| {
                        if let manifest::Feature::Project(p) = f {
                            full_path.join(p).canonicalize().unwrap() == full_pkg_path
                        } else {
                            false
                        }
                    }) {
                        continue 'dep_loop;
                    }

                    if let Some(features) = features {
                        let features: Box<dyn Iterator<Item = &String>> = if default_features {
                            Box::new(features.iter().chain(dep_manifest.features.default.iter()))
                        } else {
                            Box::new(features.iter())
                        };

                        for feature in features {
                            log::debug!("Enabling feature {} of {}", feature, name);
                            let feature_idents =
                                enable_feature((relative_path, full_path), feature, shared)?;

                            proj.dependencies
                                .extend(feature_idents.into_iter().map(|i| (i, String::from("*"))));

                            dep_proj
                                .project
                                .processor_macros
                                .insert(format!("FEATURE_{}", feature.to_uppercase()));
                        }

                        dep_proj.save()?;
                    }

                    continue 'dep_loop;
                }
            }
        }

        log::error!("{} missing dependency {}", pkg_name, name);
    }

    if is_pkg {
        shared.ws.projects.insert(
            pkg_ident.clone(),
            beef::ProjectEntry {
                path: pkg_path.0.to_path_buf(),
                ..Default::default()
            },
        );
        shared.ws.locked.insert(pkg_ident.clone());
        shared.ws_package_folder.insert(pkg_ident.clone());
    } else {
        shared.ws.projects.insert(
            pkg_name.to_owned(),
            beef::ProjectEntry {
                path: pkg_path.0.to_path_buf(),
                ..Default::default()
            },
        );
    }

    proj.save()?;

    Ok(pkg_ident)
}

fn enable_feature(
    path: (&Path, &Path),
    feature: &str,
    shared: &mut SharedConnectData,
) -> Result<Vec<String>> {
    let manifest = Manifest::from_pkg(path.1)?;

    if !manifest.features.optional.contains_key(feature) {
        bail!("Unkown feature '{}' for {}", feature, manifest.package.name);
    }

    let mut idents = Vec::new();
    match manifest.features.optional.get(feature).unwrap() {
        manifest::Feature::List(sub_features) => {
            for sub_feature in sub_features {
                enable_feature(path, sub_feature, shared)?;
            }
        }
        manifest::Feature::Project(feature_path) => {
            let ident = format!(
                "{}-{}/{}",
                manifest.package.name, manifest.package.version, feature
            );
            connect(
                &ident,
                None,
                (&path.0.join(feature_path), &path.1.join(feature_path)),
                true,
                shared,
            )?;
            idents.push(ident);
        }
    }

    Ok(idents)
}

struct SharedConnectData<'a> {
    pub pkgs: &'a Packages,
    pub ws: &'a mut beef::BeefSpace,
    pub ws_package_folder: &'a mut HashSet<String>,
    pub connects: &'a mut HashMap<PathBuf, String>,
}
