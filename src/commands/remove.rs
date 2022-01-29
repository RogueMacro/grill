use crate::{paths, prelude::*};

pub fn cli() -> App {
    App::new("remove")
        .about("Remove a package")
        .arg(Arg::new("pkg").required(true))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let pkg = args.value_of("pkg").unwrap();
    let path = paths::pkg(pkg);
    rm_rf::ensure_removed(path)?;
    Ok(())
}
