use crate::{dir, prelude::*, Manifest};

// use fs_extra::dir::CopyOptions;
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, path::Path};
use url::Url;

pub fn cli() -> App {
    subcommand("install")
        .about("Install a package into BeefLibs (can be added to a project through the IDE)")
        .arg(Arg::with_name("pkg"))
        .arg(Arg::with_name("git").long("git").value_name("URL"))
        .arg(
            Arg::with_name("path")
                .help("Path to the workspace whose dependencies would be installed")
                .long("path")
                .value_name("PATH")
                .conflicts_with_all(&["pkg", "git"]),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let (url, rev) = if let Some(url) = args.value_of("git") {
        (url.to_string(), None)
    } else if let Some(name) = args.value_of("pkg") {
        let index = crate::ops::parse_index(true, false)?;
        if !index.packages.contains_key(name) {
            crate::commands::update::exec(&ArgMatches::default())?;
        }

        if let Some(entry) = index.packages.get(name) {
            let latest = entry
                .versions
                .iter()
                .reduce(|a, b| if a > b { a } else { b })
                .with_context(|| "No versions found for this package")?;

            (entry.url.to_string(), Some(latest.1.clone()))
        } else {
            bail!("Could not find package '{}'", name);
        }
    } else {
        let path = Path::new(args.value_of("path").or(Some("./")).unwrap());
        crate::ops::install_deps(path, true)?;
        return Ok(());
    };

    let url = Url::parse(&url)?;

    rm_rf::remove(dir::tmp())?;

    let cli_progress = ProgressBar::new(1)
        .with_style(
            ProgressStyle::default_bar()
                .template("{prefix:>12.bright.green} {msg} [{bar:40}]")
                .progress_chars("=> "),
        )
        .with_prefix("Installing")
        .with_message(url.as_str().to_owned());
    cli_progress.tick();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|git_progress| {
        cli_progress.set_length(git_progress.total_objects() as u64);
        cli_progress.set_position(git_progress.indexed_objects() as u64);
        std::thread::sleep(std::time::Duration::from_millis(200));
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);
    let repo = match builder.clone(url.as_str(), &dir::tmp()) {
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

    // Dropping the builder grants us access to the directory
    drop(builder);

    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = dir::tmp().join("Package.toml");
    let mut already_installed_prompt =
        "This package is already installed, do you want to update it?".to_owned();
    let has_manifest = if let Ok(file) = fs::read_to_string(manifest_path) {
        let manifest: Manifest = toml::from_str(&file)?;
        pkg = manifest.package.name;

        let pkg_path = dir::beeflib(&pkg);
        if pkg_path.exists() {
            let installed_manifest: Manifest =
                toml::from_str(&fs::read_to_string(&pkg_path.join("Package.toml"))?)?;

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

    let pkg_path = dir::beeflib(&pkg);
    if !pkg_path.exists() {
        fs::rename(dir::tmp(), &pkg_path)?;

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
                        if dir::beeflib(input).exists() {
                            Err("A package by that name already exists")
                        } else {
                            Ok(())
                        }
                    })
                    .interact()?;

            fs::rename(&pkg_path, dir::beeflib(new_name))?;
        }
    } else if dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(already_installed_prompt)
        .interact()?
    {
        rm_rf::remove(&pkg_path)?;
        fs::rename(dir::tmp(), dir::beeflib(&pkg))?;

        println!("{} {}", console::style("Updated").bright().green(), pkg);
    }

    Ok(())
}
