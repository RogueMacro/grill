use std::fs;

use git2::Repository;

use crate::{dir, prelude::*};

pub fn cli() -> App {
    subcommand("update")
        .about("Updates the local registry with the latest packages and versions")
        .arg(Arg::with_name("quiet").long("quiet").short("q"))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    rm_rf::ensure_removed(dir::tmp())?;

    let progress = indicatif::ProgressBar::new_spinner();

    if !args.is_present("quiet") {
        progress.set_style(indicatif::ProgressStyle::default_spinner().tick_chars("|/-\\-✔"));
        progress.set_message(format!(
            "{} index",
            console::style("Updating").bright().green()
        ));
        progress.enable_steady_tick(50);
    }

    Repository::clone("https://github.com/RogueMacro/grill-index", dir::tmp())?;
    fs::copy(dir::tmp().join("index.toml"), dir::index())?;

    progress.finish();

    Ok(())
}
