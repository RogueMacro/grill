use std::{fs, path::Path};

use console::style;
use semver::{Comparator, Op, VersionReq};

use crate::{
    index, lock,
    manifest::{Dependency, Manifest},
    prelude::*,
};

pub fn cli() -> App {
    App::new("add")
        .about("Add the latest version of a package(s) to the manifest")
        .arg(
            Arg::new("packages")
                .value_name("PACKAGE")
                .required(true)
                .multiple_values(true),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let packages: Vec<&String> = args
        .get_many("packages")
        .expect("Packages need to be specified")
        .collect();

    let index = index::parse(true, false)?;
    let mut manifest = Manifest::from_pkg(".")?;

    let mut already_added = false;
    for package in packages {
        if manifest.dependencies.contains_key(package) {
            already_added = true;
            log::info!("{} is already specified in the manifest.", package);
            continue;
        }

        let entry = index.get(package).with_context(|| {
            format!(
                "'{}' could not be found. If you know this package exists, try updating the index with $ grill update --index", 
                package
            )
        })?;

        let latest = entry
            .versions
            .keys()
            .max()
            .expect("No version found for this package");

        let mut req = VersionReq::default();
        req.comparators.push(Comparator {
            op: Op::Caret,
            major: latest.major,
            minor: Some(latest.minor),
            patch: Some(latest.patch),
            pre: latest.pre.clone(),
        });

        manifest
            .dependencies
            .insert(package.clone(), Dependency::Simple(req));
    }

    fs::write(crate::paths::MANIFEST_FILENAME, toml::to_string(&manifest)?)?;
    lock::generate(Path::new("."), true, true)?;

    if already_added {
        println!();
        log::info!(
            "Update packages with the {} command.",
            style("update").yellow()
        );
    }

    Ok(())
}
