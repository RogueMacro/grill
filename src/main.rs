use anyhow::{bail, Result};

fn main() -> Result<()> {
    let args = grill::cli().get_matches();
    match args.subcommand() {
        (cmd, Some(args)) => match cmd {
            "install" => grill::commands::install::exec(args)?,
            "list" => grill::commands::list::exec(args)?,
            "remove" => grill::commands::remove::exec(args)?,
            _ => bail!("Unkown command: {}", cmd),
        },
        _ => grill::cli().print_help()?,
    }

    Ok(())
}
