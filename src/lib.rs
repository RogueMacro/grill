pub mod commands;
pub mod dir;
pub mod ops;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use clap::AppSettings;
use prelude::App;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn cli() -> App {
    App::new("grill")
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(commands::add::cli())
        .subcommand(commands::install::cli())
        .subcommand(commands::list::cli())
        .subcommand(commands::remove::cli())
        .subcommand(commands::update::cli())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Index {
    pub packages: HashMap<String, IndexEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexEntry {
    pub url: Url,
    pub versions: HashMap<Version, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, VersionReq>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BeefSpace {
    pub file_version: u32,
    #[serde(default)]
    pub locked: HashSet<String>,
    pub projects: HashMap<String, ProjectListEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ProjectListEntry {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Workspace {
    pub startup_project: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BeefProj {
    pub file_version: u32,
    pub dependencies: HashMap<String, String>,
    pub project: Project,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    pub name: String,
    pub startup_object: String,
}

pub mod prelude {
    pub type App = clap::App<'static, 'static>;

    pub use anyhow::{anyhow, bail, Context, Result};
    pub use clap::{Arg, ArgMatches};

    pub fn subcommand(name: &str) -> App {
        clap::SubCommand::with_name(name)
    }
}
