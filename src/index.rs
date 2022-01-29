use std::{collections::HashMap, fs};

use anyhow::{Context, Result};
use git2::Repository;
use indicatif::ProgressBar;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::paths;

pub type Index = HashMap<String, IndexEntry>;

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexEntry {
    pub url: Url,
    pub versions: HashMap<Version, VersionMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionMetadata {
    pub rev: String,
    pub deps: HashMap<String, VersionReq>,
}

pub fn update(with_spinner: bool, clear_after: bool) -> Result<()> {
    rm_rf::ensure_removed(paths::tmp())?;

    let spinner = ProgressBar::new_spinner();
    if with_spinner {
        spinner.set_message(format!(
            "{:>10} index",
            console::style("Updating").bright().cyan()
        ));
        spinner.enable_steady_tick(100);
    }

    Repository::clone("https://github.com/RogueMacro/grill-index", paths::tmp())?;
    fs::copy(paths::tmp().join("index.toml"), paths::index())?;

    if with_spinner {
        if clear_after {
            spinner.finish_and_clear();
        } else {
            spinner.finish_with_message(format!(
                "{:>10} index",
                console::style("Updated").bright().green()
            ));
        }
    }

    Ok(())
}

pub fn parse(with_spinner: bool, clear_after: bool) -> Result<Index> {
    let path = paths::index();
    toml::from_str::<Index>(&fs::read_to_string(&path)?).or_else(|err| {
        update(with_spinner, clear_after)?;
        toml::from_str::<Index>(&fs::read_to_string(&path)?).context(err)
    })
}
