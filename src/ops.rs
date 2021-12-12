use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use git2::Repository;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Url;
use semver::Version;

use crate::{dir, prelude::*, Index, Manifest};

pub fn update_index(with_spinner: bool, clear_after: bool) -> Result<()> {
    rm_rf::ensure_removed(dir::tmp())?;

    let spinner = ProgressBar::new_spinner();
    if with_spinner {
        spinner.set_style(ProgressStyle::default_spinner().tick_chars("|/-\\-✔"));
        spinner.set_message(format!(
            "{} index",
            console::style("Updating").bright().green()
        ));
        spinner.enable_steady_tick(100);
    }

    Repository::clone("https://github.com/RogueMacro/grill-index", dir::tmp())?;
    fs::copy(dir::tmp().join("index.toml"), dir::index())?;

    if clear_after {
        spinner.finish_and_clear();
    } else {
        spinner.finish();
    }

    Ok(())
}

pub fn parse_index(with_spinner: bool, clear_after: bool) -> Result<Index> {
    let path = dir::index();
    toml::from_str::<Index>(&fs::read_to_string(&path)?).or_else(|err| {
        update_index(with_spinner, clear_after)?;
        toml::from_str::<Index>(&fs::read_to_string(&path)?).context(err)
    })
}

pub fn install<S>(pkg: S, version: Option<&Version>) -> Result<PathBuf>
where
    S: AsRef<str>,
{
    let pkg = pkg.as_ref();

    if let Some(version) = version {
        let path = dir::pkg(format!("{}-{}", pkg, version));
        if path.exists() {
            return Ok(path);
        }
    }

    let (url, rev) = if let Ok(url) = Url::parse(pkg) {
        (url, None)
    } else {
        let index = parse_index(false, false)?;
        if !index.packages.contains_key(pkg) {
            crate::commands::update::exec(&ArgMatches::default())?;
        }

        if let Some(entry) = index.packages.get(pkg) {
            let rev = version
                .and_then(|version| {
                    entry
                        .versions
                        .get(version)
                        .and_then(|s| Some(s.clone()))
                        .or_else(|| {
                            update_index(false, true).unwrap();
                            let index = parse_index(false, false).unwrap();
                            index
                                .packages
                                .get(pkg)
                                .and_then(|entry| entry.versions.get(version))
                                .and_then(|s| Some(s.clone()))
                        })
                })
                .or_else(|| {
                    entry
                        .versions
                        .iter()
                        .reduce(|a, b| if a > b { a } else { b })
                        .and_then(|kvp| Some(kvp.1.clone()))
                })
                .with_context(|| "No versions found for this package")?;

            (entry.url.clone(), Some(rev.clone()))
        } else {
            bail!("Could not find package '{}'", pkg);
        }
    };

    rm_rf::ensure_removed(dir::tmp())?;
    let repo = Repository::clone(url.as_str(), &dir::tmp())?;
    if let Some(rev) = rev {
        let (object, reference) = repo.revparse_ext(&rev)?;
        repo.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => repo.set_head(gref.name().with_context(|| "Invalid gref name")?),
            None => repo.set_head_detached(object.id()),
        }?;
    }

    // Dropping the repository grants us access to the directory
    drop(repo);

    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = dir::tmp().join("Package.toml");
    if let Ok(file) = fs::read_to_string(&manifest_path) {
        let manifest: Manifest = toml::from_str(&file)?;

        pkg = format!(
            "{}-{}",
            manifest.package.name,
            manifest.package.version.to_string()
        );
    }

    let path = dir::pkg(&pkg);
    fs::rename(dir::tmp(), &path)?;

    Ok(path)
}

pub fn install_deps(path: &Path, with_progress: bool) -> Result<HashMap<String, PathBuf>> {
    let manifest: Manifest = toml::from_str(
        &fs::read_to_string(path.join("Package.toml"))
            .with_context(|| "No manifest file in current directory")?,
    )?;

    let pkgs = crate::ops::get_pkgs()?;
    let mut paths = HashMap::new();
    if with_progress {
        let progress = MultiProgress::new();
        let install_progress = progress.add(
            ProgressBar::new_spinner().with_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:>11}> {msg}")
                    .tick_chars("=\\|/==="),
            ),
        );
        let deps_progress = progress.add(
            ProgressBar::new(manifest.dependencies.len() as u64)
                .with_style(
                    ProgressStyle::default_bar()
                        .template("{prefix:>12.bright.cyan} [{bar:40}] {pos}/{len}")
                        .progress_chars("=> "),
                )
                .with_prefix("Working"),
        );
        install_progress.enable_steady_tick(150);
        deps_progress.tick();

        for (dep, version) in manifest.dependencies.iter() {
            if pkgs
                .get(dep)
                .and_then(|versions| Some(!versions.contains(version)))
                .unwrap_or(true)
            {
                install_progress.set_message(format!("{} v{}", dep, version));

                let path = crate::ops::install(dep, Some(version))?;
                paths.insert(dep.clone(), path);

                install_progress.println(format!(
                    "{:>12} {} v{}",
                    console::style("Installed").bright().green(),
                    dep,
                    version
                ));
            }

            deps_progress.inc(1);
        }

        deps_progress.finish();
        install_progress.finish_and_clear();
        progress.clear()?;
    } else {
        for (dep, version) in manifest.dependencies.iter() {
            if pkgs
                .get(dep)
                .and_then(|versions| Some(!versions.contains(version)))
                .unwrap_or(true)
            {
                let path = crate::ops::install(dep, Some(version))?;
                paths.insert(dep.clone(), path);
            }
        }
    }

    Ok(paths)
}

pub fn get_pkgs() -> Result<HashMap<String, HashSet<Version>>> {
    let mut pkgs = HashMap::new();
    for dir in fs::read_dir(dir::pkgs())? {
        if let Some((pkg, version)) = dir?
            .file_name()
            .to_str()
            .with_context(|| "Invalid filename")?
            .split_once(' ')
        {
            if let Ok(version) = Version::parse(version) {
                pkgs.entry(pkg.to_string())
                    .or_insert(HashSet::new())
                    .insert(version);
            }
        }
    }

    Ok(pkgs)
}
