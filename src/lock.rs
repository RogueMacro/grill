use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::Result;
use semver::{Version, VersionReq};

use crate::{manifest::Manifest, resolver::resolve};

pub type Lock = HashMap<String, HashSet<Version>>;

pub fn is_corrupt(lock: &Lock) -> bool {
    for (_, vset) in lock {
        for v in vset.iter() {
            if vset.iter().any(|v2| v.major == v2.major) {
                return true;
            }
        }
    }

    return false;
}

pub fn validate(pkg_path: &Path) -> Result<bool> {
    let deps = Manifest::from_pkg(pkg_path)?.dependencies;

    let file_path = pkg_path.join(crate::paths::LOCK_FILE);
    if !file_path.exists() {
        return Ok(false);
    }

    let lock: Lock = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(validate_lock(&deps, &lock))
}

fn validate_lock(deps: &HashMap<String, VersionReq>, lock: &Lock) -> bool {
    log::trace!("Validation lock");
    log::trace!("Dependencies: {:#?}", deps);
    log::trace!("Lock: {:#?}", lock);

    if deps.len() != lock.len() {
        log::trace!("Length doesn't match");
        return false;
    }

    for (dep, req) in deps {
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

    let resolved = resolve(&manifest, previous_lock.as_ref())?;

    let lock_file = toml::to_string(&resolved)?;
    fs::write(pkg_path.join(crate::paths::LOCK_FILE), lock_file)?;

    Ok(resolved)
}
