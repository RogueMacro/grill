use std::{
    io::{self, prelude::*},
    path::Path,
};

use anyhow::{bail, Result};
use indicatif::ProgressBar;

use super::{beefbuild, make};
use crate::manifest::Manifest;

pub fn rebuild<P>(path: P, progress: Option<&ProgressBar>) -> Result<()>
where
    P: AsRef<Path>,
{
    let manifest = Manifest::from_pkg(&path)?;
    if let Some(buildscript) = manifest.buildscript {
        let buildscript_path = path.as_ref().join(buildscript.path);

        if let Some(progress) = progress {
            progress.set_prefix(console::style("Make").bright().cyan().to_string());
        }

        make(&buildscript_path, true)?;

        let output = if let Some(progress) = progress {
            progress.set_prefix(console::style("Compile").bright().cyan().to_string());
            beefbuild::build(&buildscript_path)?;
            progress.set_prefix(console::style("Build").bright().cyan().to_string());
            beefbuild::run(&buildscript_path)?
        } else {
            beefbuild::run(&buildscript_path)?
        };

        if !output.status.success() {
            io::stdout().write_all(&output.stdout)?;
            if let Some(exit_code) = output.status.code() {
                bail!("Buildscript returned a non-zero exit code: {}", exit_code);
            } else {
                bail!("Buildscript failed.");
            }
        }
    }

    Ok(())
}
