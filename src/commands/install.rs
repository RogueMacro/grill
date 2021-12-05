use crate::{dir, prelude::*, Manifest};

use git2::Repository;
use std::fs;
use url::Url;

pub fn cli() -> App {
    subcommand("install")
        .about("Install a package")
        // .arg(Arg::with_name("pkg"))
        .arg(Arg::with_name("git").value_name("URL"))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let url = args.value_of("git").ok_or(anyhow!(
        "There is no public registry yet, so the only way to download packages are with git urls."
    ))?;

    let url = Url::parse(url)?;

    // let pkg = args.value_of("pkg").unwrap();
    println!("Installing {}", url);

    rm_rf::remove(dir::tmp())?;

    match Repository::clone(url.as_str(), dir::tmp()) {
        Ok(repo) => repo,
        Err(e) => bail!("Download failed: {}", e),
    };

    let mut pkg = url.host().ok_or(anyhow!("No host in url"))?.to_string();
    pkg.push_str(&url.path().replace("/", "-").replace(".git", ""));

    let manifest_path = dir::tmp().join("Grill.toml");
    if let Ok(file) = fs::read_to_string(manifest_path) {
        let manifest: Manifest = toml::from_str(&file)
            .with_context(|| format!("Failed to parse manifest:\n{}", &file))?;

        pkg = format!("{}-{}", manifest.package.owner, manifest.package.name);
    }

    if dir::pkg(&pkg).exists()
        && dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("This package is already installed, do you want to update it?")
            .interact()?
    {
        println!("Updating it");
    } else {
        // fs::create_dir_all(&pkg_path)?;
        // fs::copy(dir::tmp(), pkg_path)?;
        let tmp_pkg_path = dir::home().join(&pkg);
        fs::rename(dir::tmp(), &tmp_pkg_path)?;
        fs_extra::dir::move_dir(
            tmp_pkg_path,
            dir::pkgs(),
            &fs_extra::dir::CopyOptions::default(),
        )?;
    }

    Ok(())
}
