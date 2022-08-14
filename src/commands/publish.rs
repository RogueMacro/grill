use std::{collections::HashMap, fs};

use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use git2::{Oid, Repository};
use semver::VersionReq;
use serde_json::json;

use crate::{index, manifest::Manifest, paths, prelude::*};

pub fn cli() -> App {
    App::new("publish")
        .about("Publishes the current package version")
        .arg(
            Arg::new("rev")
                .short('r')
                .help("The id of the commit to publish"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let access_token = fs::read_to_string(paths::token())?;
    let manifest = Manifest::from_pkg("./")?;
    let package = manifest.package.name.clone();
    let version = manifest.package.version.to_string();

    let repo = Repository::open(".")?;
    let commit = if let Some(rev) = args.value_of("rev") {
        let commit = repo
            .find_commit(Oid::from_str(rev)?)
            .context("Commit doesn't exist")?;

        commit
    } else {
        let head = repo.head()?;
        let rev = head.target().context("Could not get HEAD target")?;
        repo.find_commit(rev)?
    };
    let rev = commit.id().to_string();

    println!("{:>12} {}", style("Package").bright().yellow(), package);
    println!("{:>12} {}", style("Version").bright().yellow(), version);
    println!("{:>12} {}", style("Commit").bright().yellow(), style(&rev));
    if let Some(msg) = commit.message() {
        println!(
            "{:>14} {} {}",
            style('"').italic(),
            msg.trim(),
            style('"').italic(),
        );
    }
    println!();

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Continue?")
        .interact()?
    {
        println!();
        index::update(true, true)?;
        let index = index::parse(false, false)?;
        let mut body = json!({
            "access_token": access_token,
            "package": package.clone(),
            "metadata": json!({
                "version": version.clone(),
                "revision": rev,
                "dependencies": manifest.deps_with_req().collect::<HashMap<&String, &VersionReq>>()
            })
        });
        if !index.contains_key(&package) {
            let remote_urls: Vec<String> = repo
                .remotes()?
                .iter()
                .map(|remote| {
                    repo.find_remote(remote.unwrap())
                        .unwrap()
                        .url()
                        .and_then(|url| Some(url.to_owned()))
                })
                .filter_map(|url| url)
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

        let res = crate::web::api("publish", &body)?;

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
