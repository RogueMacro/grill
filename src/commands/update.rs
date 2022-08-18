use std::path::Path;

use crate::prelude::*;

pub fn cli() -> App {
    App::new("update")
        .about("Update dependencies to the latest version")
        .arg(Arg::new("quiet").long("quiet").short('q'))
        .arg(Arg::new("grill").long("grill").help("not yet implemented"))
        .arg(
            Arg::new("index")
                .long("index")
                .help("Update the package index"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    if args.is_present("grill") {
        todo!()
    } else if args.is_present("index") {
        crate::index::update(!args.is_present("quiet"), false)
    } else {
        crate::index::update(!args.is_present("quiet"), false)?;
        crate::lock::generate(Path::new("."), true)?;
        Ok(())
    }
}
