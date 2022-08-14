use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::Result;
use semver::Version;

use crate::{manifest::Manifest, resolver::resolve};

pub type Lock = HashMap<String, HashSet<Version>>;

pub fn is_corrupt(lock: &Lock) -> bool {
    for (_, versions) in lock {
        for v in versions.iter() {
            if versions.iter().any(|v2| v.major == v2.major) {
                return true;
            }
        }
    }

    return false;
}

pub fn validate(pkg_path: &Path) -> Result<bool> {
    let manifest = Manifest::from_pkg(pkg_path)?;

    let file_path = pkg_path.join(crate::paths::LOCK_FILE);
    if !file_path.exists() {
        return Ok(false);
    }

    let lock: Lock = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(validate_lock(&manifest, &lock))
}

fn validate_lock<'a>(manifest: &Manifest, lock: &Lock) -> bool {
    log::trace!("Validation lock");
    log::trace!("Dependencies: {:#?}", manifest.dependencies);
    log::trace!("Lock: {:#?}", lock);

    if manifest.dependencies.len() != lock.len() {
        log::trace!("Length doesn't match");
        return false;
    }

    for (dep, req) in manifest.deps_with_req() {
        if !lock
            .get(dep)
            .map_or(false, |vset| vset.iter().any(|v| req.matches(v)))
        {
            log::error!("No match for {} {} in vset {:?}", dep, req, lock.get(dep));
            return false;
        }
    }

    true
}

pub fn generate(pkg_path: &Path) -> Result<Lock> {
    let manifest = Manifest::from_pkg(pkg_path)?;

    let lock_path = pkg_path.join(crate::paths::LOCK_FILE);
    let previous_lock = if lock_path.exists() {
        let lock = toml::from_str(&fs::read_to_string(lock_path)?)?;
        if is_corrupt(&lock) {
            None
        } else {
            Some(lock)
        }
    } else {
        None
    };

    let index = crate::index::parse(false, false)?;

    let resolved = resolve(&manifest, previous_lock.as_ref(), &index)?;

    let lock_file = toml::to_string(&resolved)?;
    fs::write(pkg_path.join(crate::paths::LOCK_FILE), lock_file)?;

    Ok(resolved)
}
