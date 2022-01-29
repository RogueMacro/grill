use std::fs;

use crate::{paths, prelude::*};
use dialoguer::{theme::ColorfulTheme, Input};
use serde::{Deserialize, Serialize};

pub fn cli() -> App {
    App::new("login").about("Login through the CLI")
}

pub fn exec(_: &ArgMatches) -> Result<()> {
    let token = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("paste your token here (found on Account > Settings > Authorization)\n")
        .interact()?;

    fs::write(paths::token(), token)?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorBody {
    pub message: String,
}
