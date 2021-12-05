use crate::{dir, prelude::*};

use std::fs;

pub fn cli() -> App {
    subcommand("list").about("List all installed packages")
}

pub fn exec(_args: &ArgMatches) -> Result<()> {
    for entry in fs::read_dir(dir::pkgs())? {
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
