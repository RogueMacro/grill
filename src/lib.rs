pub mod commands;
pub mod dir;
pub mod ops;

use std::collections::HashMap;

use clap::AppSettings;
use prelude::App;
use semver::Version;
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
    pub dependencies: HashMap<String, Version>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
}

pub mod prelude {
    pub type App = clap::App<'static, 'static>;

    pub use anyhow::{anyhow, bail, Context, Result};
    pub use clap::{Arg, ArgMatches};

    pub fn subcommand(name: &str) -> App {
        clap::SubCommand::with_name(name)
    }
}
