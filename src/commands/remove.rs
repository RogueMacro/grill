use crate::{dir, prelude::*};

pub fn cli() -> App {
    subcommand("remove")
        .about("Remove a package")
        .arg(Arg::with_name("pkg").required(true))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let pkg = args.value_of("pkg").unwrap();
    let path = dir::pkg(pkg);
    rm_rf::ensure_removed(path)?;
    Ok(())
}
