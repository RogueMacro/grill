pub mod beef;
pub mod commands;
pub mod index;
pub mod lock;
pub mod log;
pub mod manifest;
pub mod ops;
pub mod paths;
pub mod resolver;
pub mod webapi;

use prelude::{App, Arg};

pub fn cli() -> App {
    App::new("grill")
        .about(clap::crate_description!())
        .version(clap::crate_version!())
        .arg(Arg::new("debug").long("debug").global(true))
        .subcommand(commands::add::cli())
        .subcommand(commands::init::cli())
        .subcommand(commands::install::cli())
        .subcommand(commands::list::cli())
        .subcommand(commands::login::cli())
        .subcommand(commands::make::cli())
        .subcommand(commands::new::cli())
        .subcommand(commands::publish::cli())
        .subcommand(commands::purge::cli())
        .subcommand(commands::rebuild::cli())
        .subcommand(commands::update::cli())
}

pub mod prelude {
    pub type App = clap::App<'static>;

    pub use anyhow::{anyhow, bail, Context, Result};
    pub use clap::{Arg, ArgMatches};
}
