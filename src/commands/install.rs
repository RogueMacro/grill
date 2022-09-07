use std::fs;

use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use indicatif::{ProgressBar, ProgressStyle};
use url::Url;

use crate::{index, manifest::Manifest, paths, prelude::*};

pub fn cli() -> App {
    App::new("install")
        .about("Install a package into BeefLibs (can be added to a project through the IDE)")
        .arg(Arg::new("pkg").required_unless_present("git"))
        .arg(
            Arg::new("git")
                .long("git")
                .value_name("URL")
                .required_unless_present("pkg"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let (url, rev) = if let Some(url) = args.value_of("git") {
        (url.to_string(), None)
    } else {
        let pkg = args.value_of("pkg").unwrap();

        let index = index::parse(true, false)?;
        if !index.contains_key(pkg) {
            index::update(true, false)?;
        }

        if let Some(entry) = index.get(pkg) {
            let latest = entry
                .versions
                .iter()
                .reduce(|a, b| if a.0 > b.0 { a } else { b })
                .with_context(|| "No versions found for this package")?;

            (entry.url.to_string(), Some(latest.1.rev.clone()))
        } else {
            bail!("Could not find package '{}'", pkg);
        }
    };

    let url = Url::parse(&url)?;

    rm_rf::remove(paths::tmp())?;

    let cli_progress = ProgressBar::new(1)
        .with_style(
            ProgressStyle::default_bar()
                .template("{prefix:>12.bright.green} {msg} [{bar:40}]")?
                .progress_chars("=> "),
        )
        .with_prefix("Installing")
        .with_message(url.as_str().to_owned());
    cli_progress.tick();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|git_progress| {
        cli_progress.set_length(git_progress.total_objects() as u64);
        cli_progress.set_position(git_progress.indexed_objects() as u64);
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);
    let repo = match builder.clone(url.as_str(), &paths::tmp()) {
        Ok(repo) => repo,
        Err(e) => bail!("Download failed: {}", e),
    };
    if let Some(rev) = rev {
        let (object, reference) = repo.revparse_ext(&rev)?;
        repo.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => repo.set_head(gref.name().with_context(|| "Invalid gref name")?),
            None => repo.set_head_detached(object.id()),
        }?;
    }

    // Dropping the repo grants us access to the directory
    drop(repo);

    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = paths::tmp().join(crate::paths::MANIFEST_FILENAME);
    let mut already_installed_prompt =
        "This package is already installed, do you want to update it?".to_owned();
    let has_manifest = if let Ok(file) = fs::read_to_string(manifest_path) {
        let manifest = Manifest::from_file(&file)?;
        pkg = manifest.package.name;

        let pkg_path = paths::beeflib(&pkg);
        if pkg_path.exists() {
            let installed_manifest = Manifest::from_pkg(pkg_path)?;
            if installed_manifest.package.version == manifest.package.version {
                return Ok(());
            }

            already_installed_prompt = format!(
                "This package is already installed, do you want to update it from v{} to v{}?",
                installed_manifest.package.version, manifest.package.version
            );
        }

        true
    } else {
        false
    };

    let pkg_path = paths::beeflib(&pkg);
    if !pkg_path.exists() {
        fs::rename(paths::tmp(), &pkg_path)?;

        if !has_manifest
            && dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt(format!(
                    "Installed package as {}. Do you want to rename it?",
                    pkg
                ))
                .interact()?
        {
            let new_name: String =
                dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Enter a new name".to_owned())
                    .validate_with(|input: &String| {
                        if paths::beeflib(input).exists() {
                            Err("A package by that name already exists")
                        } else {
                            Ok(())
                        }
                    })
                    .interact()?;

            fs::rename(&pkg_path, paths::beeflib(new_name))?;
        }
    } else if dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(already_installed_prompt)
        .interact()?
    {
        rm_rf::remove(&pkg_path)?;
        fs::rename(paths::tmp(), &pkg_path)?;

        println!("{} {}", console::style("Updated").bright().green(), pkg);
    }

    Ok(())
}
