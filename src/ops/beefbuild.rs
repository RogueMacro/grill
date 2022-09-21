use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Context, Result};

pub fn run<P>(path: P) -> Result<Output>
where
    P: AsRef<Path>,
{
    build(&path)?;

    let mut command = create_command(path)?;
    command.arg("-run");
    Ok(command.output()?)
}

pub fn build<P>(path: P) -> Result<Output>
where
    P: AsRef<Path>,
{
    Ok(create_command(path)?.output()?)
}

fn create_command<P>(path: P) -> Result<Command>
where
    P: AsRef<Path>,
{
    let beefpath =
        env::var("BeefPath").context("BeefPath not found. Are you sure Beef is installed?")?;
    let mut exe = PathBuf::from(beefpath);
    exe.push("bin");
    exe.push("BeefBuild");

    let mut command = Command::new(exe);
    command.arg(format!("-workspace={}", path.as_ref().to_string_lossy()));
    Ok(command)
}
