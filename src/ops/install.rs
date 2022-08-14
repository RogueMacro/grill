use std::{collections::HashMap, fs, path::PathBuf, time::Duration};

use anyhow::Context;
use git2::Repository;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Url;
use semver::Version;

use crate::{
    beef,
    index::{self, Index},
    paths,
    prelude::*,
};

pub fn install<S>(pkg: S, version: &Version, index: Option<&Index>) -> Result<PathBuf>
where
    S: AsRef<str>,
{
    let pkg = pkg.as_ref();

    let mut owned_index = None;
    let index = index.map_or_else(
        || -> Result<&Index> {
            owned_index = Some(index::parse(false, false)?);
            Ok(owned_index.as_ref().unwrap())
        },
        |i| Ok(i),
    )?;

    if !index.contains_key(pkg) {
        bail!("Could not find package '{}'", pkg);
    }

    let entry = index.get(pkg).unwrap();
    let metadata = entry
        .versions
        .get(version)
        .with_context(|| format!("{} is not a version of '{}'", version, pkg))?;

    let ident = format!("{}-{}", pkg, version);
    let path = paths::pkg(&ident);
    if path.exists() {
        return Ok(path);
    }

    let path = install_git(&entry.url, Some(&metadata.rev), Some(&ident))?;
    Ok(path)
}

pub fn install_git(url: &Url, rev: Option<&str>, pkg_ident: Option<&String>) -> Result<PathBuf> {
    rm_rf::ensure_removed(paths::tmp())?;
    let repo = Repository::clone(url.as_str(), &paths::tmp())?;
    if let Some(rev) = rev {
        let (object, reference) = repo.revparse_ext(&rev)?;
        repo.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => repo.set_head(gref.name().with_context(|| "Invalid gref name")?),
            None => repo.set_head_detached(object.id()),
        }?;
    }

    // Dropping the repository gives us access to the directory
    drop(repo);

    let path = pkg_ident.map(|ident| paths::pkg(ident)).unwrap_or_else(|| {
        let mut pkg = url
            .host()
            .ok_or(anyhow!("No host in url"))
            .unwrap()
            .to_string();
        pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));
        paths::pkg(&pkg)
    });

    if path.exists() {
        bail!("Package already installed")
    }

    fs::rename(paths::tmp(), &path).context("Failed to move package")?;

    if let Some(ident) = pkg_ident {
        let proj_file_path = path.join("BeefProj.toml");
        let mut proj_file: beef::BeefProj =
            toml::from_str(&fs::read_to_string(&proj_file_path)?)
                .with_context(|| format!("Failed to read project file of package '{}'", ident))?;

        proj_file.project.name = ident.to_owned();

        fs::write(&proj_file_path, toml::to_string(&proj_file)?)
            .context("Failed to write project file")?;
    }

    if path.join(paths::PACKAGE_FILE).exists() {
        crate::ops::make(&path, true)?;
    }

    Ok(path)
}

pub fn install_multiple<F>(
    pkgs: &HashMap<String, Version>,
    with_progress: bool,
    index: Option<&Index>,
    on_install: Option<F>,
) -> Result<HashMap<String, PathBuf>>
where
    F: Fn(&str, &Version),
{
    let mut owned_index = None;
    let index = index.map_or_else(
        || -> Result<&Index> {
            owned_index = Some(index::parse(false, false)?);
            Ok(owned_index.as_ref().unwrap())
        },
        |i| Ok(i),
    )?;

    let mut paths = HashMap::new();
    if with_progress {
        let progress = MultiProgress::new();
        let install_progress = progress.add(
            ProgressBar::new_spinner().with_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:>11}> {msg}")?
                    .tick_chars("=\\|/==="),
            ),
        );
        let fetch_progress = progress.add(
            ProgressBar::new(pkgs.len() as u64)
                .with_style(
                    ProgressStyle::default_bar()
                        .template("{prefix:>12.bright.cyan} [{bar:40}] {pos}/{len}")?
                        .progress_chars("=> "),
                )
                .with_prefix("Fetching"),
        );
        install_progress.enable_steady_tick(Duration::from_millis(150));
        fetch_progress.tick();

        for (pkg, version) in pkgs.iter() {
            install_progress.set_message(format!("{} v{}", pkg, version));

            let path = install(pkg, version, Some(index))?;
            paths.insert(pkg.clone(), path);

            install_progress.println(format!(
                "{:>12} {} v{}",
                console::style("Installed").bright().green(),
                pkg,
                version
            ));
            fetch_progress.inc(1);

            if let Some(on_install) = &on_install {
                on_install(pkg, version);
            }
        }

        fetch_progress.finish();
        install_progress.finish_and_clear();
        progress.clear()?;
    } else {
        for (pkg, version) in pkgs.iter() {
            let path = install(pkg, version, Some(index))?;
            paths.insert(pkg.clone(), path);
            if let Some(on_install) = &on_install {
                on_install(pkg, version);
            }
        }
    }

    Ok(paths)
}
