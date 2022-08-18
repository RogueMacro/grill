use std::{collections::HashMap, path::Path};

use anyhow::Context;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
}

impl Manifest {
    pub fn from_pkg<P>(path: P) -> anyhow::Result<Manifest>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let manifest = toml::from_str(
            &std::fs::read_to_string(path.join(crate::paths::MANIFEST_FILENAME))
                .context(format!("No manifest found at in '{}'", path.display()))?,
        )?;
        Ok(manifest)
    }

    /// Get simple dependencies with requirements.
    /// Ignores git dependencies.
    pub fn simple_deps(&self) -> impl Iterator<Item = (&String, &VersionReq)> {
        self.dependencies.iter().filter_map(|(key, val)| {
            if let Dependency::Simple(req) = val {
                Some((key, req))
            } else {
                None
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Package {
    pub name: String,
    pub version: Version,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Dependency {
    Simple(VersionReq),
    Git(GitDependency),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GitDependency {
    pub git: String,
    pub rev: Option<String>,
}
