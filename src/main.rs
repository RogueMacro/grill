use std::fs::File;

use anyhow::{bail, Result};
use grill::paths;

fn main() -> Result<()> {
    let console_logger = Box::new(
        env_logger::builder()
            .format_timestamp(None)
            .format_target(false)
            .build(),
    );
    let file_logger = simplelog::WriteLogger::new(
        log::LevelFilter::max(),
        Default::default(),
        File::create(paths::home().join("log.txt"))?,
    );
    multi_log::MultiLogger::init(vec![console_logger, file_logger], log::Level::Trace)?;

    rm_rf::ensure_removed(paths::tmp())?;

    let result = {
        let args = grill::cli().get_matches();
        match args.subcommand() {
            Some((cmd, args)) => match cmd {
                "install" => grill::commands::install::exec(args),
                "list" => grill::commands::list::exec(args),
                "login" => grill::commands::login::exec(args),
                "make" => grill::commands::make::exec(args),
                "publish" => grill::commands::publish::exec(args),
                "purge" => grill::commands::purge::exec(args),
                "update" => grill::commands::update::exec(args),
                _ => bail!("Unkown command: {}", cmd),
            },
            None => {
                grill::cli().print_help()?;
                Ok(())
            }
        }
    };

    if let Err(err) = result {
        println!();
        if let Some(source) = err.source() {
            log::error!("{}\n\nCaused by:\n    {}", err, source);
        } else {
            log::error!("{}", err);
        }
        println!();
    }

    rm_rf::ensure_removed(paths::tmp())?;

    Ok(())
}
