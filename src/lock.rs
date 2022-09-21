use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use semver::Version;

use crate::{manifest::Manifest, resolver::resolve};

pub type Lock = HashMap<String, HashSet<Version>>;

pub fn read<P>(path: P) -> Result<Lock>
where
    P: AsRef<Path>,
{
    toml::from_str(&fs::read_to_string(path.as_ref()).with_context(|| {
            format!(
                "Failed to read lock file: {}",
                path.as_ref().to_string_lossy()
            )
        })?)
        .context("Failed to deserialize lock file")
}

pub fn write<P>(path: P, lock: &Lock) -> Result<()>
where
    P: AsRef<Path>,
{
    fs::write(
        path,
        toml::to_string(lock).context("Failed to serialize lock")?,
    )
    .context("Failed to write lock file")
}

pub fn validate(pkg_path: &Path) -> Result<bool> {
    let manifest = Manifest::from_pkg(pkg_path)?;

    let file_path = pkg_path.join(crate::paths::LOCK_FILENAME);
    if !file_path.exists() {
        return Ok(false);
    }

    let lock: Lock = self::read(file_path)?;
    Ok(validate_lock(&manifest, &lock))
}

fn validate_lock(manifest: &Manifest, lock: &Lock) -> bool {
    log::trace!("Validating lock");

    for (dep, req) in manifest.simple_deps() {
        if !lock.get(dep).map_or(false, |locked_versions| {
            locked_versions.iter().any(|v| req.matches(v))
        }) {
            log::trace!("Invalid lock: No match for {} {}", dep, req);
            return false;
        }
    }

    for (dep, versions) in lock {
        for v1 in versions.iter() {
            for v2 in versions.iter() {
                if v1.major == v2.major && v1 != v2 {
                    log::trace!(
                        "Invalid lock: {} v{} is incompatible with {} v{}",
                        dep,
                        v1,
                        dep,
                        v2
                    );
                    return false;
                }
            }
        }
    }

    true
}

pub fn generate(pkg_path: &Path, write_lock: bool, try_keep_lock: bool) -> Result<Lock> {
    let manifest = Manifest::from_pkg(pkg_path).context("Failed to read manifest")?;

    let lock_path = pkg_path.join(crate::paths::LOCK_FILENAME);
    let previous_lock = if try_keep_lock && lock_path.exists() {
        let lock = self::read(lock_path)?;
        if validate_lock(&manifest, &lock) {
            Some(lock)
        } else {
            None
        }
    } else {
        None
    };

    let index = crate::index::parse(false, false)?;

    let lock = resolve(&manifest, previous_lock.as_ref(), &index)?;

    if write_lock {
        self::write(pkg_path.join(crate::paths::LOCK_FILENAME), &lock)?;
    }

    Ok(lock)
}
