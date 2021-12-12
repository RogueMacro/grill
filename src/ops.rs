use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use git2::Repository;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Url;
use semver::{Version, VersionReq};

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

pub fn install<S>(pkg: S, req: &VersionReq) -> Result<(PathBuf, Version)>
where
    S: AsRef<str>,
{
    let pkg = pkg.as_ref();

    let mut index = parse_index(false, false)?;
    if !index.packages.contains_key(pkg) {
        update_index(false, false)?;
        index = parse_index(false, false)?;
        if !index.packages.contains_key(pkg) {
            bail!("Could not find package '{}'", pkg);
        }
    }

    let entry = index.packages.get(pkg).unwrap();
    let mut versions = entry.versions.iter().collect::<Vec<(&Version, &String)>>();
    versions.sort_by(|a, b| a.0.cmp(b.0));
    let mut req_match = None;
    for (version, version_rev) in versions.iter().rev() {
        if req.matches(version) {
            req_match = Some((version.clone(), version_rev.clone()));
            break;
        }
    }

    let (version, rev) = req_match.ok_or_else(|| {
        anyhow!(
            "No version found for package '{}' that matches the requirement '{}'",
            pkg,
            req
        )
    })?;

    let path = dir::pkg(format!("{}-{}", pkg, version));
    if path.exists() {
        return Ok((path, version.clone()));
    }

    let path = install_git(&entry.url, Some(&rev))?;
    Ok((path, version.clone()))
}

pub fn install_git(url: &Url, rev: Option<&str>) -> Result<PathBuf> {
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
    let mut deps = HashMap::new();
    if let Ok(file) = fs::read_to_string(&manifest_path) {
        let manifest: Manifest = toml::from_str(&file)?;

        pkg = format!(
            "{}-{}",
            manifest.package.name,
            manifest.package.version.to_string()
        );

        deps = manifest.dependencies;
    }

    let path = dir::pkg(&pkg);
    fs::rename(dir::tmp(), &path)?;

    for (dep, req) in deps {
        install(dep, &req)?;
    }

    Ok(path)
}

pub fn install_deps(path: &Path, with_progress: bool) -> Result<HashMap<String, PathBuf>> {
    let manifest: Manifest = toml::from_str(
        &fs::read_to_string(path.join("Package.toml"))
            .context("No manifest file in current directory")?,
    )?;

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

        for (dep, req) in manifest.dependencies.iter() {
            install_progress.set_message(format!("{} v{}", dep, req));

            let (path, version) = crate::ops::install(dep, req)?;
            paths.insert(dep.clone(), path);

            install_progress.println(format!(
                "{:>12} {} v{}",
                console::style("Installed").bright().green(),
                dep,
                version
            ));
            deps_progress.inc(1);
        }

        deps_progress.finish();
        install_progress.finish_and_clear();
        progress.clear()?;
    } else {
        for (dep, version) in manifest.dependencies.iter() {
            let (path, _) = crate::ops::install(dep, version)?;
            paths.insert(dep.clone(), path);
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
