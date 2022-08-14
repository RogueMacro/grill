use std::fs;

use crate::prelude::*;

pub fn cli() -> App {
    App::new("purge").about("Deletes all installed packages")
}

pub fn exec(_: &ArgMatches) -> Result<()> {
    let path = crate::paths::pkgs();
    let count = fs::read_dir(&path)?.count();

    if count == 0 {
        println!("No packages in cache");
        return Ok(());
    }

    if !dialoguer::Confirm::new()
        .with_prompt(format!(
            "Are you sure you want to delete {} packages?",
            count
        ))
        .interact()?
    {
        return Ok(());
    }

    rm_rf::remove(&path)?;
    println!(
        "{:>12} {} packages",
        console::style("Deleted").bright().red(),
        count
    );

    Ok(())
}
