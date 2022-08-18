use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::Result;
use semver::Version;

use crate::{manifest::Manifest, resolver::resolve};

pub type Lock = HashMap<String, HashSet<Version>>;

pub fn validate(pkg_path: &Path) -> Result<bool> {
    let manifest = Manifest::from_pkg(pkg_path)?;

    let file_path = pkg_path.join(crate::paths::LOCK_FILENAME);
    if !file_path.exists() {
        return Ok(false);
    }

    let lock: Lock = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(validate_lock(&manifest, &lock))
}

fn validate_lock<'a>(manifest: &Manifest, lock: &Lock) -> bool {
    log::trace!("Validating lock");

    if manifest.dependencies.len() != lock.len() {
        log::trace!("Length doesn't match");
        return false;
    }

    for (dep, req) in manifest.simple_deps() {
        if !lock.get(dep).map_or(false, |locked_versions| {
            locked_versions.iter().any(|v| req.matches(v))
        }) {
            log::trace!("No match for {} {} in lock", dep, req);
            return false;
        }
    }

    for (_, versions) in lock {
        for v in versions.iter() {
            if versions.iter().any(|v2| v.major == v2.major) {
                return false;
            }
        }
    }

    true
}

pub fn generate(pkg_path: &Path, write_lock: bool) -> Result<Lock> {
    let manifest = Manifest::from_pkg(pkg_path)?;

    let lock_path = pkg_path.join(crate::paths::LOCK_FILENAME);
    let previous_lock = if lock_path.exists() {
        let lock = toml::from_str(&fs::read_to_string(lock_path)?)?;
        if validate_lock(&manifest, &lock) {
            Some(lock)
        } else {
            None
        }
    } else {
        None
    };

    let index = crate::index::parse(false, false)?;

    let resolved = resolve(&manifest, previous_lock.as_ref(), &index)?;

    if write_lock {
        let lock_file = toml::to_string(&resolved)?;
        fs::write(pkg_path.join(crate::paths::LOCK_FILENAME), lock_file)?;
    }

    Ok(resolved)
}
