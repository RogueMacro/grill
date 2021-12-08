use std::iter;

use anyhow::{bail, Result};
use grill::dir;

fn main() -> Result<()> {
    let args = grill::cli().get_matches();

    if args.subcommand_name() != Some("update") && !dir::index().exists() {
        grill::commands::update::exec(
            &grill::cli().get_matches_from(iter::empty::<std::ffi::OsString>()),
        )?;
    }

    match args.subcommand() {
        (cmd, Some(args)) => match cmd {
            "add" => grill::commands::add::exec(args)?,
            "install" => grill::commands::install::exec(args)?,
            "list" => grill::commands::list::exec(args)?,
            "remove" => grill::commands::remove::exec(args)?,
            "update" => grill::commands::update::exec(args)?,
            _ => bail!("Unkown command: {}", cmd),
        },
        _ => grill::cli().print_help()?,
    }

    Ok(())
}
