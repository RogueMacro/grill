use std::{fs, path::Path};

use crate::{beef::BeefProj, prelude::*};

pub fn cli() -> App {
    App::new("new")
        .about("Create a new workspace and project")
        .arg(Arg::new("path").value_name("PATH").default_value("."))
        .arg(
            Arg::new("lib")
                .long("lib")
                .help("Set project to be a library"),
        )
        .arg(
            Arg::new("gui")
                .long("gui")
                .help("Set project to be a GUI application"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let path = Path::new(args.value_of("path").unwrap());
    if !path.exists() {
        fs::create_dir(path)?;
    }

    let name = fs::canonicalize(std::env::current_dir()?.join(path))?
        .file_name()
        .context("Invalid filename")?
        .to_string_lossy()
        .to_string();

    fs::create_dir(path.join("src"))?;
    fs::write(
        path.join("src").join("Program.bf"),
        format!(
            "\
using System;

namespace {}
{{
    class Program
    {{
        public static int Main(String[] args)
        {{
            return 0;
        }}
    }}
}}
    ",
            name
        ),
    )?;

    crate::ops::init::init(path, &name)?;
    crate::ops::make::make(path, false)?;

    if args.is_present("lib") {
        let mut proj = BeefProj::from_file(&path.join("BeefProj.toml"))?;
        proj.project.target_type = String::from("BeefLib");
        proj.save()?;
    } else if args.is_present("gui") {
        let mut proj = BeefProj::from_file(&path.join("BeefProj.toml"))?;
        proj.project.target_type = String::from("BeefGUIApplication");
        proj.save()?;
    }

    Ok(())
}
