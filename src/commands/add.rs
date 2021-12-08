use std::path::PathBuf;

use crate::prelude::*;

pub fn cli() -> App {
    subcommand("add")
        .about("Add a package to a workspace")
        .arg(Arg::with_name("pkg").required(true))
        .arg(
            Arg::with_name("path")
                .long("path")
                .value_name("PATH")
                .default_value(".")
                .help("Path to the workspace"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let path = PathBuf::from(args.value_of("path").unwrap());

    Ok(())
}
