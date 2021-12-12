use crate::{dir, prelude::*};

use std::fs;

pub fn cli() -> App {
    subcommand("list").about("List all installed packages").arg(
        Arg::with_name("themes")
            .long("themes")
            .help("List installed themes"),
    )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let dir = if args.is_present("themes") {
        dir::themes()
    } else {
        dir::pkgs()
    };

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        println!(
            "{}",
            path.file_name()
                .ok_or(anyhow!("No filename"))?
                .to_string_lossy()
        );
    }

    Ok(())
}
