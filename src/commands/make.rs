use std::path::PathBuf;

use crate::prelude::*;

pub fn cli() -> App {
    App::new("make")
        .about("Install the neccessary dependencies and make a workspace")
        .arg(
            Arg::new("path")
                .long("path")
                .value_name("PATH")
                .default_value(".")
                .help("Path to the workspace"),
        )
        .arg(Arg::new("quiet").long("quiet").short('q'))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let path = PathBuf::from(args.value_of("path").unwrap());
    crate::ops::make(&path, args.is_present("quiet"))
}
