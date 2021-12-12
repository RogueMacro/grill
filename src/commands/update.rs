use crate::prelude::*;

pub fn cli() -> App {
    subcommand("update")
        .about("Updates the local registry with the latest packages and versions")
        .arg(Arg::with_name("quiet").long("quiet").short("q"))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    crate::ops::update_index(!args.is_present("quiet"), false)
}
