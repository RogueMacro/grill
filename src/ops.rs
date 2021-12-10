use std::{
    collections::{HashMap, HashSet},
    fs,
};

use git2::Repository;
use reqwest::Url;
use semver::Version;

use crate::{dir, prelude::*, Index, Manifest};

pub fn install<S>(pkg: S, version: Option<&Version>) -> Result<()>
where
    S: AsRef<str>,
{
    let pkg = pkg.as_ref();

    if let Some(version) = version {
        if dir::pkg(format!("{}-{}", pkg, version)).exists() {
            return Ok(());
        }
    }

    let (url, rev) = if let Ok(url) = Url::parse(pkg) {
        (url, None)
    } else {
        let index: Index = toml::from_str(&fs::read_to_string(dir::index())?)?;
        if !index.packages.contains_key(pkg) {
            crate::commands::update::exec(&ArgMatches::default())?;
        }

        if let Some(entry) = index.packages.get(pkg) {
            let rev = version
                .and_then(|version| entry.versions.get(version))
                .or_else(|| {
                    entry
                        .versions
                        .iter()
                        .reduce(|a, b| if a > b { a } else { b })
                        .and_then(|kvp| Some(kvp.1))
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
    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = dir::tmp().join("Grill.toml");
    if let Ok(file) = fs::read_to_string(&manifest_path) {
        let manifest: Manifest =
            toml::from_str(&file).with_context(|| format!("Failed to parse manifest"))?;

        pkg = format!(
            "{}-{}",
            manifest.package.name,
            manifest.package.version.to_string()
        );
    }

    if dir::pkg(&pkg).exists() {
        return Ok(());
    }

    fs::rename(dir::tmp(), dir::pkgs().join(&pkg))?;

    Ok(())
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
