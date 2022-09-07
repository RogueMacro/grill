use std::{fs, path::Path};

use crate::{beef::BeefProj, prelude::*};

pub fn cli() -> App {
    App::new("init")
        .about("Initialize an existing beef workspace with grill")
        .arg(
            Arg::new("path")
                .long("path")
                .value_name("PATH")
                .default_value("."),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let path = Path::new(args.value_of("path").unwrap());
    let proj_path = path.join("BeefProj.toml");
    let name = if proj_path.exists() {
        BeefProj::from_file(&proj_path)?.project.name
    } else {
        fs::canonicalize(std::env::current_dir()?.join(path))?
            .file_name()
            .context("Invalid filename")?
            .to_string_lossy()
            .to_string()
    };

    crate::ops::init::init(path, &name)
}
