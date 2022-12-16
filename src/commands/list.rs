use std::fs;

use crate::{paths, prelude::*};

pub fn cli() -> App {
    App::new("list").about("List all installed packages").arg(
        Arg::new("themes")
            .long("themes")
            .help("List installed themes"),
    )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let dir = if args.is_present("themes") {
        paths::themes()
    } else {
        paths::pkgs(".")
    };

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        println!(
            "{}",
            path.file_name()
                .ok_or_else(|| anyhow!("No filename"))?
                .to_string_lossy()
        );
    }

    Ok(())
}
