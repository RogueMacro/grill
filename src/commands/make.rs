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
        .arg(
            Arg::new("fix-pkg")
                .long("fix")
                .value_name("PACKAGE")
                .help("Run make for an installed package")
                .conflicts_with("path"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    if let Some(pkg) = args.value_of("fix-pkg") {
        let pkg_path = crate::paths::pkg(args.value_of("path").unwrap(), pkg);
        if !pkg_path.exists() {
            bail!("Package '{}' is not installed. Did you include the right version? I.e. Dummy-1.2.3", pkg)
        }
        log::debug!("Fixing {}", pkg);
        crate::ops::install::prepare_pkg(&pkg_path, None)
    } else {
        let path = PathBuf::from(args.value_of("path").unwrap());
        crate::ops::make(&path, args.is_present("quiet"))
    }
}
