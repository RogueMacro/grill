use crate::prelude::*;

pub fn cli() -> App {
    App::new("update-index")
        .about("Updates the local registry with the latest packages and versions")
        .arg(Arg::new("quiet").long("quiet").short('q'))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    crate::index::update(!args.is_present("quiet"), false)
}
