use std::{collections::HashMap, path::Path};

use anyhow::Context;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, VersionReq>,
}

impl Manifest {
    pub fn from_pkg<P>(path: P) -> anyhow::Result<Manifest>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let manifest = toml::from_str(
            &std::fs::read_to_string(path.join(crate::paths::PACKAGE_FILE))
                .context(format!("No manifest found at in '{}'", path.display()))?,
        )?;
        Ok(manifest)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
}
