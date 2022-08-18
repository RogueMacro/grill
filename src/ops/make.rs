use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    time::Duration,
};

use anyhow::Context;
use console::Emoji;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    beef,
    index::{self},
    lock::{self, Lock},
    manifest::Manifest,
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
            "{:>12} {} v{}\n",
            console::style("Make").bright().cyan(),
            manifest.package.name,
            manifest.package.version
        );
    }

    let multi = MultiProgress::new();

    let index = make_step(
        &multi,
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
        &multi,
        2,
        4,
        "Resolving",
        "Resolution ready",
        &LOOKING_GLASS,
        silent,
        |_, _| {
            if !lock::validate(&path)? {
                lock::generate(&path, false)
            } else {
                let lock: Lock =
                    toml::from_str(&fs::read_to_string(path.join(crate::paths::LOCK_FILENAME))?)
                        .context("Invalid lock")?;
                Ok(lock)
            }
        },
    )?;

    let pkgs = make_step(
        &multi,
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
                        .template("{prefix:>12.bright.cyan} [{bar:11}]")?
                        .progress_chars("=> "),
                ),
            );
            if !silent {
                progress.set_length(
                    lock.iter()
                        .map(|(_, versions)| versions.len())
                        .sum::<usize>() as u64,
                );
                progress.set_style(
                    ProgressStyle::default_bar()
                        .template("{msg:>12} [{bar:11}]")?
                        .progress_chars("=> "),
                );
                progress.set_message(format!(
                    "{} / {}",
                    0,
                    progress.length().ok_or(anyhow!("No progress length"))?
                ));
            }

            let mut pkgs = HashMap::new();
            for (pkg, versions) in lock {
                for version in versions {
                    let path = crate::ops::install(&pkg, &version, Some(&index))?;
                    pkgs.insert(format!("{}-{}", pkg, version), path);

                    if !silent {
                        progress.inc(1);
                        progress.set_message(format!(
                            "{} / {}",
                            progress.position(),
                            progress.length().ok_or(anyhow!("No progress length"))?
                        ));
                    }
                }
            }

            progress.finish_and_clear();
            Ok(pkgs)
        },
    )?;

    make_step(
        &multi,
        4,
        4,
        "Building",
        "Workspace done",
        &PACKAGE,
        silent,
        |_, _| {
            let ws_file_path = path.join("BeefSpace.toml");
            let proj_file_path = path.join("BeefProj.toml");
            let mut ws: beef::BeefSpace = toml::from_str(&fs::read_to_string(&ws_file_path)?)?;
            let mut proj: beef::BeefProj = toml::from_str(&fs::read_to_string(&proj_file_path)?)?;

            if !ws.workspace_folders.contains_key("Packages") {
                ws.workspace_folders
                    .insert(String::from("Packages"), HashSet::new());
            }

            let ws_package_folder = ws.workspace_folders.get_mut("Packages").unwrap();
            ws.projects.retain(|proj, _| {
                pkgs.contains_key(proj) || manifest.dependencies.contains_key(proj)
            });
            ws_package_folder.retain(|pkg| pkgs.contains_key(pkg));
            ws.projects.insert(
                String::from("corlib"),
                beef::ProjectEntry {
                    path: crate::paths::beeflib("corlib"),
                    ..Default::default()
                },
            );
            ws.locked.insert(String::from("corlib"));

            proj.dependencies
                .retain(|pkg, _| manifest.dependencies.contains_key(pkg));
            proj.dependencies
                .insert(String::from("corlib"), String::from("*"));

            for (ident, path) in pkgs {
                ws.projects.insert(
                    ident.clone(),
                    beef::ProjectEntry {
                        path,
                        ..Default::default()
                    },
                );

                ws.locked.insert(ident.clone());

                ws_package_folder.insert(ident.clone());

                if manifest
                    .dependencies
                    .contains_key(ident.rsplit_once('-').unwrap().0)
                {
                    proj.dependencies.insert(ident, String::from("*"));
                }
            }

            fs::write(&ws_file_path, toml::to_string(&ws)?)?;
            fs::write(&proj_file_path, toml::to_string(&proj)?)?;

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
        step as usize - 1,
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
