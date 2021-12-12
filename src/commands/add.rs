use std::{fs, path::PathBuf};

use crate::{prelude::*, BeefSpace, ProjectListEntry};

pub fn cli() -> App {
    subcommand("add")
        .about("Adds dependencies to a workspace")
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

    let beefspace_path = path.join("BeefSpace.toml");
    let mut beefspace: BeefSpace = toml::from_str(
        &fs::read_to_string(&beefspace_path).with_context(|| "No BeefSpace found")?,
    )?;

    let dep_paths = crate::ops::install_deps(&path, true)?;
    for (dep, path) in dep_paths {
        beefspace
            .projects
            .insert(dep.clone(), ProjectListEntry { path });
        beefspace.locked.insert(dep);
    }

    let ser = toml::to_string(&beefspace)?;
    fs::write(beefspace_path, ser)?;

    Ok(())
}
