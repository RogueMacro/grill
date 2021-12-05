pub mod commands;
pub mod dir;

use clap::AppSettings;
use prelude::App;
use serde::{Deserialize, Serialize};

pub fn cli() -> App {
    App::new("grill")
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(commands::add::cli())
        .subcommand(commands::install::cli())
        .subcommand(commands::list::cli())
        .subcommand(commands::remove::cli())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub package: Package,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub owner: String,
}

pub mod prelude {
    pub type App = clap::App<'static, 'static>;

    pub use anyhow::{anyhow, bail, Context, Result};
    pub use clap::{Arg, ArgMatches};

    pub fn subcommand(name: &str) -> App {
        clap::SubCommand::with_name(name)
    }
}
