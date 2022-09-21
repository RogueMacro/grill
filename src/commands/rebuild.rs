use std::{borrow::Cow, fs, path::Path};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{paths, prelude::*};

pub fn cli() -> App {
    App::new("rebuild")
        .about("Run the Build script for this package")
        .arg(
            Arg::new("all")
                .long("all")
                .short('a')
                .help("Rebuild all packages")
                .conflicts_with("pkgs"),
        )
        .arg(
            Arg::new("pkgs")
                .multiple_values(true)
                .help("Rebuild the specified packages"),
        )
        .arg(Arg::new("quiet").long("quiet").short('q'))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let quiet = args.is_present("quiet");

    if args.is_present("all") {
        for path in fs::read_dir(paths::pkgs("."))? {
            rebuild(&path?.path(), quiet)?;
        }
    } else if let Some(mut pkgs) = args.values_of("pkgs") {
        for dir in fs::read_dir(paths::pkgs("."))? {
            let dir = dir?;
            let dir_name = dir.file_name();
            let ident = dir_name.to_string_lossy();
            let (pkg, _) = ident.rsplit_once('-').context("Invalid file name")?;

            if pkgs.any(|v| v == ident || v == pkg) {
                rebuild(&dir.path(), quiet)?;
            }
        }
    }

    rebuild(Path::new("."), quiet)
}

fn rebuild(path: &Path, quiet: bool) -> Result<()> {
    let file_name = if path.ends_with(".") {
        Cow::Owned(
            std::env::current_dir()?
                .file_name()
                .context("Invalid file name")?
                .to_string_lossy()
                .to_string(),
        )
    } else {
        path.file_name()
            .context("Invalid file name")?
            .to_string_lossy()
    };

    let spinner = if !quiet {
        let spinner = ProgressBar::new_spinner()
            .with_message(format!("{}", file_name))
            .with_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:>12} {msg} {spinner}")?
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈✔"),
            );
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(spinner)
    } else {
        None
    };

    crate::ops::rebuild(path, spinner.as_ref())?;

    if let Some(spinner) = spinner {
        spinner.set_prefix(console::style("Finished").bright().green().to_string());
        spinner.finish();
    }

    Ok(())
}
