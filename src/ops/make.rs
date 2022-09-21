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

const COMPASS: Emoji = Emoji("🧭 ", "");
const LOOKING_GLASS: Emoji = Emoji("🔍 ", "");
const TRUCK: Emoji = Emoji("🚚 ", "");
const PACKAGE: Emoji = Emoji("📦 ", "");
const SPAGHETTI: Emoji = Emoji("🍝 ", "");

pub fn make(path: &Path, silent: bool) -> Result<()> {
    let manifest = Manifest::from_pkg(&path)?;

    if !silent {
        println!(
            "{:>12} {} v{}",
            console::style("Make").bright().cyan(),
            manifest.package.name,
            manifest.package.version
        );
    }

    let multi = crate::log::get_multi_progress();

    // Invisible progress bar to create empty line between logs and progress bars
    let p = multi.add(ProgressBar::new(0).with_style(ProgressStyle::default_bar().template(" ")?));
    p.finish();

    let index = make_step(
        multi,
        1,
        4,
        "Updating",
        "Up to date",
        &COMPASS,
        silent,
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
        silent,
        |_, _| {
            if !lock::validate(path)? {
                lock::generate(path, true, true)
            } else {
                lock::read(path.join(crate::paths::LOCK_FILENAME))
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
        silent,
        |multi, _| {
            let progress = multi.add(
                ProgressBar::new(1).with_style(
                    ProgressStyle::default_bar()
                        .template("{prefix:>12} [{bar:40}] {msg:.bright.grey}")?
                        .progress_chars("=> "),
                ),
            );
            if !silent {
                progress.set_length(
                    lock.iter()
                        .map(|(_, versions)| versions.len())
                        .sum::<usize>() as u64,
                );
                progress.set_prefix(format!(
                    "{} / {}",
                    0,
                    progress.length().ok_or(anyhow!("No progress length"))?
                ));
            }

            let mut pkgs = HashMap::new();
            for (pkg, versions) in lock {
                for version in versions {
                    progress.set_message(format!("{} 0%", pkg));
                    let (path, fetched) =
                        crate::ops::install(&pkg, &version, Some(&index), |install_progress| {
                            progress.set_message(format!(
                                "{} {}%",
                                pkg,
                                (install_progress.indexed_objects() as f32
                                    / install_progress.total_objects() as f32
                                    * 100f32)
                                    .floor(),
                            ))
                        })?;

                    pkgs.insert((pkg.clone(), either::Left(version.clone())), path);

                    if !silent {
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
                            progress.length().ok_or(anyhow!("No progress length"))?
                        ));
                    }
                }
            }

            for (name, dep) in manifest.git_deps() {
                progress.set_message(format!("{} 0%", name));
                let (path, rev) = crate::ops::install_git(
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

                pkgs.insert((name.clone(), either::Right(rev)), path);

                if !silent {
                    progress.inc(1);
                    progress.set_prefix(format!(
                        "{} / {}",
                        progress.position(),
                        progress.length().ok_or(anyhow!("No progress length"))?
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
        silent,
        |_, _| {
            let ws_file_path = path.join("BeefSpace.toml");
            let proj_file_path = path.join("BeefProj.toml");

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

            connect(
                &manifest.package.name,
                Some(&either::Left(manifest.package.version)),
                path,
                &pkgs,
                &mut ws,
                &mut ws_package_folder,
                false,
            )?;

            ws.projects.get_mut(&manifest.package.name).unwrap().path = ".".into();

            for ((pkg_name, pkg_version), pkg_path) in pkgs.iter() {
                connect(
                    pkg_name,
                    Some(pkg_version),
                    pkg_path,
                    &pkgs,
                    &mut ws,
                    &mut ws_package_folder,
                    true,
                )?;
            }

            ws.workspace_folders
                .insert(String::from("Packages"), ws_package_folder);
            fs::write(&ws_file_path, toml::to_string(&ws)?)?;

            Ok(())
        },
    )?;

    if !silent {
        println!("\n{:>13}{}Enjoy your spaghetti!", " ", SPAGHETTI);
    }

    Ok(())
}

fn make_step<F, T>(
    multi: &MultiProgress,
    step: i32,
    steps: i32,
    msg: &'static str,
    finish: &'static str,
    emoji: &Emoji,
    silent: bool,
    func: F,
) -> Result<T>
where
    F: FnOnce(&MultiProgress, &ProgressBar) -> Result<T>,
{
    let spinner_style = ProgressStyle::default_spinner()
        .template("{msg} {spinner}")?
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈✔");
    let progress = multi.insert(
        step as usize, /*- 1*/
        ProgressBar::new_spinner()
            .with_message(format!(
                "{:>12} {}{}",
                console::style(format!("[{}/{}]", step, steps)).dim(),
                emoji,
                msg
            ))
            .with_style(spinner_style),
    );
    if !silent {
        progress.enable_steady_tick(Duration::from_millis(100));
    }

    let result = func(multi, &progress)?;

    if !silent {
        progress.finish_with_message(format!(
            "{:>12} {}{}",
            console::style(format!("[{}/{}]", step, steps)).bold().dim(),
            emoji,
            finish
        ));
    }

    Ok(result)
}

fn connect(
    pkg_name: &str,
    pkg_version: Option<&Either<Version, String>>,
    pkg_path: &Path,
    pkgs: &HashMap<(String, Either<Version, String>), PathBuf>,
    ws: &mut beef::BeefSpace,
    ws_package_folder: &mut HashSet<String>,
    is_pkg: bool,
) -> Result<String> {
    let manifest = Manifest::from_pkg(&pkg_path)?;
    let mut proj = beef::BeefProj::from_file(&pkg_path.join("BeefProj.toml"))?;
    proj.dependencies.clear();
    proj.dependencies
        .insert(String::from("corlib"), String::from("*"));

    let pkg_ident = if let Some(pkg_version) = pkg_version {
        match pkg_version {
            either::Left(v) => format!("{}-{}", pkg_name, v),
            either::Right(rev) => format!("{}-{}", pkg_name, rev),
        }
    } else {
        pkg_name.to_owned()
    };

    'dep_loop: for (name, dep) in manifest.dependencies.iter() {
        if let manifest::Dependency::Local(local) = dep {
            let dep_path = pkg_path.join(&local.path);
            let dep_manifest = Manifest::from_pkg(&dep_path)?;

            let dep_ident = if std::fs::canonicalize(&dep_path)?.starts_with(pkg_path) {
                // We are a root package
                connect(
                    &format!("{}/{}", pkg_name, name),
                    None,
                    &dep_path,
                    pkgs,
                    ws,
                    ws_package_folder,
                    true,
                )?
            } else {
                // We are a package inside a package
                connect(
                    name,
                    Some(&either::Left(dep_manifest.package.version)),
                    &dep_path,
                    pkgs,
                    ws,
                    ws_package_folder,
                    true,
                )?
            };

            proj.dependencies.insert(dep_ident, String::from("*"));

            continue;
        }

        for ((pkg, version), path) in pkgs.iter() {
            if pkg == name {
                let mut features = None;
                let mut default_features = false;
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
                    let ident = match version {
                        either::Left(v) => format!("{}-{}", pkg, v),
                        either::Right(rev) => format!("{}-{}", pkg, rev),
                    };
                    proj.dependencies.insert(ident, String::from("*"));

                    if let Some(features) = features {
                        let dep_manifest = Manifest::from_pkg(path)?;

                        let features: Box<dyn Iterator<Item = &String>> = if default_features {
                            Box::new(features.iter().chain(dep_manifest.features.default.iter()))
                        } else {
                            Box::new(features.iter())
                        };

                        for feature in features {
                            let feature_idents =
                                enable_feature(path, feature, ws, ws_package_folder, pkgs)?;
                            proj.dependencies
                                .extend(feature_idents.into_iter().map(|i| (i, String::from("*"))));
                        }
                    }

                    continue 'dep_loop;
                }
            }
        }

        log::error!("{} missing dependency {}", pkg_name, name);
    }

    if is_pkg {
        ws.projects.insert(
            pkg_ident.clone(),
            beef::ProjectEntry {
                path: pkg_path.to_path_buf(),
                ..Default::default()
            },
        );
        ws.locked.insert(pkg_ident.clone());
        ws_package_folder.insert(pkg_ident.clone());
    } else {
        ws.projects.insert(
            pkg_name.to_owned(),
            beef::ProjectEntry {
                path: pkg_path.to_path_buf(),
                ..Default::default()
            },
        );
    }

    proj.save()?;

    Ok(pkg_ident)
}

fn enable_feature(
    path: &Path,
    feature: &str,
    ws: &mut beef::BeefSpace,
    ws_package_folder: &mut HashSet<String>,
    pkgs: &HashMap<(String, Either<Version, String>), PathBuf>,
) -> Result<Vec<String>> {
    let manifest = Manifest::from_pkg(path)?;

    if !manifest.features.optional.contains_key(feature) {
        bail!("Unkown feature '{}' for {}", feature, manifest.package.name);
    }

    let mut idents = Vec::new();
    match manifest.features.optional.get(feature).unwrap() {
        manifest::Feature::List(sub_features) => {
            for sub_feature in sub_features {
                enable_feature(path, sub_feature, ws, ws_package_folder, pkgs)?;
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
                &path.join(feature_path),
                pkgs,
                ws,
                ws_package_folder,
                true,
            )?;
            idents.push(ident);
        }
    }

    Ok(idents)
}
