use std::path::Path;

use crate::{
    index,
    lock::{self, Lock},
    prelude::*,
};
use console::style;

pub fn cli() -> App {
    App::new("update")
        .about("Update dependencies to the latest version")
        .arg(Arg::new("quiet").long("quiet").short('q'))
        // .arg(
        //     Arg::new("grill")
        //         .long("grill")
        //         .help("not yet implemented")
        //         .conflicts_with("index"),
        // )
        .arg(
            Arg::new("index")
                .long("index")
                .help("Update the package index")
                .conflicts_with("grill"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    if args.is_present("grill") {
        todo!()
    } else if args.is_present("index") {
        index::update(!args.is_present("quiet"), false)
    } else {
        index::update(!args.is_present("quiet"), false)?;

        let lock_path = Path::new(".").join(crate::paths::LOCK_FILENAME);
        let old_lock = if lock_path.exists() {
            Some(lock::read(lock_path)?)
        } else {
            None
        };

        let lock = lock::generate(Path::new("."), true, false)?;

        if !args.is_present("quiet") {
            if let Some(old_lock) = old_lock {
                print_altered_deps(&old_lock, &lock);
            } else {
                for (dep, versions) in lock {
                    for version in versions {
                        println!(
                            "{:>12} {} v{}",
                            style("Added").bright().cyan(),
                            dep,
                            version
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

fn print_altered_deps(old_lock: &Lock, new_lock: &Lock) {
    for (dep, versions) in new_lock {
        for version in versions {
            if old_lock.get(dep).map_or(true, |old_versions| {
                !old_versions.iter().any(|v| v.major == version.major)
            }) {
                println!(
                    "{:>12} {} v{}",
                    style("Added").bright().cyan(),
                    dep,
                    version
                );
            }
        }
    }

    for (dep, new_versions) in new_lock {
        for new_version in new_versions {
            if let Some(old_version) = old_lock.get(dep).and_then(|old_versions| {
                old_versions
                    .iter()
                    .find(|&v| v.major == new_version.major && v != new_version)
            }) {
                println!(
                    "{:>12} {} v{} -> v{}",
                    style("Updated").bright().green(),
                    dep,
                    old_version,
                    new_version
                );
            }
        }
    }

    for (dep, versions) in old_lock {
        for version in versions {
            if !new_lock.get(dep).map_or(false, |new_versions| {
                new_versions.iter().any(|v| v.major == version.major)
            }) {
                println!(
                    "{:>12} {} v{}",
                    style("Removed").bright().red(),
                    dep,
                    version
                );
            }
        }
    }
}
