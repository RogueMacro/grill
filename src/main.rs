use anyhow::{bail, Result};
use grill::paths;

fn main() -> Result<()> {
    rm_rf::ensure_removed(paths::tmp())?;

    let result = {
        let args = grill::cli().get_matches();

        if args.contains_id("debug") {
            grill::log::init(log::LevelFilter::Debug)?;
        } else {
            grill::log::init(log::LevelFilter::Info)?;
        }

        match args.subcommand() {
            Some((cmd, args)) => match cmd {
                "add" => grill::commands::add::exec(args),
                "init" => grill::commands::init::exec(args),
                "install" => grill::commands::install::exec(args),
                "list" => grill::commands::list::exec(args),
                "login" => grill::commands::login::exec(args),
                "make" => grill::commands::make::exec(args),
                "new" => grill::commands::new::exec(args),
                "publish" => grill::commands::publish::exec(args),
                "purge" => grill::commands::purge::exec(args),
                "rebuild" => grill::commands::rebuild::exec(args),
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

        if let Some(src) = err.source() {
            log::error!("{}\n\nCaused by:\n    {}", err, src);
        } else {
            log::error!("{}", err);
        }

        log::debug!("Backtrace: {}", err.backtrace());

        println!();
    }

    if !cfg!(debug_assertions) {
        rm_rf::ensure_removed(paths::tmp())?;
    }

    Ok(())
}
