use crate::{dir, prelude::*, Index, Manifest};

use fs_extra::dir::CopyOptions;
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks, Repository};
use std::fs;
use url::Url;

pub fn cli() -> App {
    subcommand("install")
        .about("Install a git repository into BeefLibs (can be accessed from IDE)")
        .arg(Arg::with_name("pkg"))
        .arg(Arg::with_name("git").long("git").value_name("URL"))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let (url, rev) = if let Some(url) = args.value_of("git") {
        (url.to_string(), None)
    } else if let Some(name) = args.value_of("pkg") {
        let index: Index = toml::from_str(&fs::read_to_string(dir::index())?)?;
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
        let manifest: Manifest = toml::from_str(
            &fs::read_to_string("./Grill.toml")
                .with_context(|| "No manifest file in current directory")?,
        )?;

        let pkgs = crate::ops::get_pkgs()?;
        let all = pkgs.len() as u64;
        let mut installed = 0;
        let progress = indicatif::ProgressBar::new(all).with_message(format!(
            "{} dependencies",
            console::style("Installing").bright().green()
        ));
        progress.tick();
        for (dep, version) in manifest.dependencies.iter() {
            if pkgs
                .get(dep)
                .and_then(|versions| Some(!versions.contains(version)))
                .unwrap_or(true)
            {
                crate::ops::install(dep, true)?;
                installed += 1;
            }

            progress.tick();
        }
        progress.finish_with_message(format!(
            "{} {}/{} dependencies",
            console::style("Installed").bright().green(),
            installed,
            all
        ));

        return Ok(());
    };

    let url = Url::parse(&url)?;

    println!("{} {}", console::style("Installing").bright().green(), url);

    rm_rf::remove(dir::tmp())?;

    let cli_progress = indicatif::ProgressBar::new(1);
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
    match builder.clone(url.as_str(), &dir::tmp()) {
        Ok(repo) => repo,
        Err(e) => bail!("Download failed: {}", e),
    };

    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = dir::tmp().join("Grill.toml");
    if let Ok(file) = fs::read_to_string(manifest_path) {
        let manifest: Manifest = toml::from_str(&file)
            .with_context(|| format!("Failed to parse manifest:\n{}", &file))?;

        pkg = manifest.package.name;
    }

    if !dir::beeflib(&pkg).exists() {
        let tmp_pkg_path = dir::home().join(&pkg);
        fs::rename(dir::tmp(), &tmp_pkg_path)?;
        match fs_extra::dir::copy(
            &tmp_pkg_path,
            dir::beeflibs(),
            &fs_extra::dir::CopyOptions::default(),
        ) {
            Ok(_) => {
                println!("Installed package as {}", pkg);
            }
            Err(e) => {
                let pkg_path = dir::beeflib(pkg);
                rm_rf::ensure_removed(pkg_path)?;
                rm_rf::ensure_removed(tmp_pkg_path)?;
                return Err(e.into());
            }
        }
    } else if dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("This package is already installed, do you want to update it?")
        .interact()?
    {
        let v1 = if let Ok(manifest) = toml::from_str::<Manifest>(
            &fs::read_to_string(dir::beeflib(&pkg).join("Grill.toml")).unwrap(),
        ) {
            Some(manifest.package.version)
        } else {
            None
        };

        let repo = Repository::open(dir::beeflib(&pkg))?;
        let remote = repo.find_remote(
            repo.branch_upstream_remote("refs/heads/main")?
                .as_str()
                .with_context(|| "Invalid remote")?,
        )?;
        let url = remote
            .url()
            .with_context(|| format!("No remote url for package '{}'", pkg))?;

        rm_rf::ensure_removed(dir::tmp())?;
        Repository::clone(url, dir::tmp())?;

        rm_rf::remove(dir::beeflib(&pkg)).unwrap();
        let tmp_pkg_path = dir::home().join(&pkg);
        fs::rename(dir::tmp(), &tmp_pkg_path)?;
        fs_extra::dir::copy(&tmp_pkg_path, dir::beeflibs(), &CopyOptions::default()).unwrap();
        rm_rf::remove(&tmp_pkg_path)?;

        let v2 = if let Ok(manifest) =
            toml::from_str::<Manifest>(&fs::read_to_string(dir::beeflib(&pkg).join("Grill.toml"))?)
        {
            Some(manifest.package.version)
        } else {
            None
        };

        println!();
        if let (Some(v1), Some(v2)) = (v1, v2) {
            println!(
                "{} {} from {} to {}",
                console::style("Updated").bright().green(),
                pkg,
                console::style(v1).bright().blue(),
                console::style(v2).bright().blue()
            );
        } else {
            println!("{} {}", console::style("Updated").bright().green(), pkg);
        }

        println!("-");
    }

    Ok(())
}
