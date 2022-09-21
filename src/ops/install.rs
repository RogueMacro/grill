use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use reqwest::Url;
use semver::Version;

use crate::{
    beef,
    index::{self, Index},
    manifest::Manifest,
    paths,
    prelude::*,
};

/// Returns the path to the installed package and a bool indicating
/// if the package was downloaded.
pub fn install<S, C>(
    ws: &Path,
    pkg: S,
    version: &Version,
    index: Option<&Index>,
    progress_callback: C,
) -> Result<(PathBuf, PathBuf, bool)>
where
    S: AsRef<str>,
    C: FnMut(git2::Progress<'_>),
{
    let pkg = pkg.as_ref();

    let mut owned_index = None;
    let index = index.map_or_else(
        || -> Result<&Index> {
            owned_index = Some(index::parse(false, false)?);
            Ok(owned_index.as_ref().unwrap())
        },
        Ok,
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
    let path = paths::pkg(ws, Path::new(&ident));
    if path.exists() {
        return Ok((paths::pkg(".", &ident), path, false));
    }

    let (relative_path, full_path, _) = install_git(
        ws,
        &entry.url,
        Some(&metadata.rev),
        Some(&ident),
        progress_callback,
    )?;

    Ok((relative_path, full_path, true))
}

/// Returns the path to the installed package, first relative to the workspace,
/// then relative to the working directory. Last is the revision that was checked out.
pub fn install_git<C>(
    ws: &Path,
    url: &Url,
    rev: Option<&str>,
    pkg_ident: Option<&String>,
    mut progress_callback: C,
) -> Result<(PathBuf, PathBuf, String)>
where
    C: FnMut(git2::Progress<'_>),
{
    rm_rf::ensure_removed(paths::tmp())?;

    // let repo = Repository::init(&paths::tmp())?;
    // let mut remote = repo.remote("origin", url.as_str())?;
    let checkout_rev;
    {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.transfer_progress(|p| {
            progress_callback(p);
            true
        });

        // remote.download::<String>(
        //     &[],
        //     Some(git2::FetchOptions::new().remote_callbacks(callbacks)),
        // )?;

        // log::trace!(
        //     "Downloaded from remote: {}",
        //     remote.url().unwrap_or("Url unavailable")
        // );
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);
        let repo = git2::build::RepoBuilder::new()
            .fetch_options(fo)
            .clone(url.as_str(), &paths::tmp())?;

        let head = repo
            .find_commit(repo.head().unwrap().target().unwrap())?
            .id()
            .to_string();

        // drop(remote);

        checkout_rev = rev.map(str::to_owned).unwrap_or(head);

        let (object, reference) = repo.revparse_ext(&checkout_rev)?;
        repo.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => repo.set_head(gref.name().with_context(|| "Invalid gref name")?),
            None => repo.set_head_detached(object.id()),
        }?;

        // Dropping the repository gives us access to the directory
    }

    let relative_path = pkg_ident
        .map(|ident| paths::pkg("", ident))
        .unwrap_or_else(|| {
            let mut pkg = url.host().unwrap().to_string();
            pkg.push_str(&url.path().replace('/', "-").replace(".git", ""));
            pkg = format!("{}-{}", pkg, checkout_rev);
            paths::pkg("", &pkg)
        });

    let full_path = ws.join(&relative_path);

    if full_path.exists() {
        return Ok((relative_path, full_path, checkout_rev));
    }

    fs::rename(paths::tmp(), &full_path).context("Failed to move tmp folder")?;

    if pkg_ident.is_some() {
        prepare_pkg(&full_path, pkg_ident.map(String::as_str))?;
    }

    Ok((relative_path, full_path, checkout_rev))
}

pub fn prepare_pkg(path: &Path, ident: Option<&str>) -> Result<()> {
    let ident = ident.map(str::to_owned).unwrap_or(
        path.file_name()
            .context("Invalid file name for package path")?
            .to_string_lossy()
            .to_string(),
    );

    let proj_file_path = path.join("BeefProj.toml");
    let mut proj_file = beef::BeefProj::from_file(&proj_file_path)?;

    proj_file.project.name = ident.clone();
    proj_file
        .dependencies
        .insert(String::from("corlib"), String::from("*"));

    let manifest = Manifest::from_pkg(&path)?;

    for (feature_name, feature_project) in
        manifest
            .features
            .optional
            .iter()
            .filter_map(|(feature_name, feature)| {
                if let crate::manifest::Feature::Project(project) = feature {
                    Some((feature_name, project))
                } else {
                    None
                }
            })
    {
        let feature_proj_path = path.join(feature_project).join("BeefProj.toml");
        let mut feature_proj_file = beef::BeefProj::from_file(&feature_proj_path)?;
        feature_proj_file.project.name = format!("{}/{}", ident, feature_name);
        feature_proj_file.save()?;
    }

    for feature_project in manifest.features.optional.iter().filter_map(|(_, f)| {
        if let crate::manifest::Feature::Project(project) = f {
            Some(project)
        } else {
            None
        }
    }) {
        let feature_project_path = path.join(feature_project);
        if !feature_project_path.join(paths::MANIFEST_FILENAME).exists() {
            continue;
        }

        let feature_manifest = Manifest::from_pkg(&feature_project_path)?;

        let feature_proj_file_path = feature_project_path.join("BeefProj.toml");
        let mut feature_proj = beef::BeefProj::from_file(&feature_proj_file_path)?;
        feature_proj.dependencies.clear();
        feature_proj
            .dependencies
            .insert(String::from("corlib"), String::from("*"));

        for (_, dep) in feature_manifest.local_deps() {
            let dep_proj_path = feature_project_path.join(&dep.path).join("BeefProj.toml");
            let dep_proj_file = beef::BeefProj::from_file(&dep_proj_path)?;

            feature_proj
                .dependencies
                .insert(dep_proj_file.project.name, String::from("*"));
        }

        feature_proj.save()?;
    }

    proj_file.save()?;

    Ok(())
}
