use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use fs_extra::dir::CopyOptions;
use git2::Repository;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::Url;
use semver::Version;

use crate::{dir, prelude::*, Index, Manifest};

pub fn install<S>(pkg: S, is_beeflib: bool, append_version: bool) -> Result<()>
where
    S: AsRef<str>,
{
    let pkg = pkg.as_ref();

    let (url, rev) = if let Ok(url) = Url::parse(pkg) {
        (url, None)
    } else {
        let index: Index = toml::from_str(&fs::read_to_string(dir::index())?)?;
        if !index.packages.contains_key(pkg) {
            crate::commands::update::exec(&ArgMatches::default())?;
        }

        if let Some(entry) = index.packages.get(pkg) {
            let latest = entry
                .versions
                .iter()
                .reduce(|a, b| if a > b { a } else { b })
                .with_context(|| "No versions found for this package")?;

            (entry.url.clone(), Some(latest.1.clone()))
        } else {
            bail!("Could not find package '{}'", pkg);
        }
    };

    // rm_rf::remove(dir::tmp())?;

    let tmp_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    // TODO: Clone directly to dest
    Repository::clone(url.as_str(), &dir::tmp())?;

    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = dir::tmp().join("Grill.toml");
    if let Ok(file) = fs::read_to_string(manifest_path) {
        let manifest: Manifest = toml::from_str(&file)
            .with_context(|| format!("Failed to parse manifest:\n{}", &file))?;

        pkg = manifest.package.name;
    }

    if is_beeflib {
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

            let v2 = if let Ok(manifest) = toml::from_str::<Manifest>(&fs::read_to_string(
                dir::beeflib(&pkg).join("Grill.toml"),
            )?) {
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
    } else {
    }

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
