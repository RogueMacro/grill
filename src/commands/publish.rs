use std::{collections::HashMap, fs};

use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use git2::{Oid, Repository};
use semver::VersionReq;
use serde_json::json;

use crate::{
    index,
    manifest::{self, Manifest},
    paths,
    prelude::*,
};

pub fn cli() -> App {
    App::new("publish")
        .about("Publish the current package version")
        .arg(
            Arg::new("commit")
                .long("commit")
                .value_name("COMMIT_HASH")
                .help("The commit hash to publish"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let token_path = paths::token();
    if !token_path.exists() {
        log::info!("You have to log in to use this command. Run `grill login` first.");
        return Ok(());
    }

    let access_token = fs::read_to_string(token_path)?;
    let manifest = Manifest::from_pkg(".")?;
    let package = manifest.package.name.clone();
    let version = manifest.package.version.to_string();

    let repo = Repository::open(".").context("Failed to open repository")?;
    let commit = if let Some(rev) = args.value_of("commit") {
        if rev.len() != "568e61d10fbcb81c3403e598b5794dd418694a58".len() {
            bail!("Commit cannot be found without long hash");
        }

        repo.find_commit(Oid::from_str(rev)?)
            .context("Commit doesn't exist")?
    } else {
        let head = repo.head()?;
        let rev = head.target().context("Could not get HEAD target")?;
        repo.find_commit(rev)?
    };
    let rev = commit.id().to_string();

    let is_dirty = repo
        .describe(git2::DescribeOptions::new().describe_all())?
        .format(Some(
            git2::DescribeFormatOptions::new().dirty_suffix("-dirty"),
        ))?
        .ends_with("-dirty");

    println!("{:>12} {}", style("Package").bright().yellow(), package);
    println!("{:>12} {}", style("Version").bright().yellow(), version);
    println!("{:>12} {}", style("Commit").bright().yellow(), style(&rev));

    if is_dirty {
        print!("{:>12}", style("(dirty)").bright().red());
    } else {
        print!("{:>12}", "");
    }

    if let Some(msg) = commit.message() {
        println!(
            " {} {} {}",
            style('"').italic(),
            msg.trim(),
            style('"').italic(),
        );
    } else {
        println!();
    }
    println!();

    let prompt = if is_dirty {
        "You have uncommitted changes. Do you still want to continue?"
    } else {
        "Publish?"
    };

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()?
    {
        println!();
        index::update(true, true)?;
        let index = index::parse(false, false)?;

        let mut deps: HashMap<String, VersionReq> = manifest
            .simple_deps()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (_, feature) in manifest.features.optional.iter() {
            if let manifest::Feature::Project(path) = feature {
                let feature_manifest = Manifest::from_pkg(path)?;
                deps.extend(
                    feature_manifest
                        .simple_deps()
                        .map(|(k, v)| (k.clone(), v.clone())),
                );
            }
        }

        let mut body = json!({
            "access_token": access_token,
            "package": package,
            "metadata": json!({
                "version": version,
                "revision": rev,
                "dependencies": deps,
                "description": manifest.package.description
            })
        });

        if !index.contains_key(&package) {
            let remote_urls: Vec<String> = repo
                .remotes()?
                .iter()
                .filter_map(|remote| {
                    repo.find_remote(remote.unwrap())
                        .unwrap()
                        .url()
                        .map(|url| url.to_owned())
                })
                .collect();

            let selected = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("This package does not exist yet, pick a remote url for your package")
                .items(&remote_urls)
                .default(0)
                .interact()?;

            body.as_object_mut().unwrap().insert(
                String::from("create_url"),
                serde_json::Value::String(remote_urls[selected].clone()),
            );
        }

        let res = crate::webapi::grill("/publish", &body)?;

        if let Err(err) = res.error_for_status_ref() {
            let body = res
                .json::<serde_json::Map<String, serde_json::Value>>()
                .unwrap();
            let server_err = if body.contains_key("statusCode") {
                anyhow!("{} (status: {})", body["message"], body["statusCode"])
            } else {
                anyhow!("{}", body["message"])
            };
            return Err(server_err).context(err);
        }

        println!(
            "   {} {} {}{}",
            style("Published").bright().green(),
            package,
            style("v").bright().blue(),
            style(version).bright().blue()
        );
    }

    Ok(())
}
