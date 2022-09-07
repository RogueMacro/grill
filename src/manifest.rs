use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Context;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub features: Features,
}

impl Manifest {
    pub fn from_pkg<P>(path: P) -> anyhow::Result<Manifest>
    where
        P: AsRef<Path>,
    {
        Self::from_file(path.as_ref().join(crate::paths::MANIFEST_FILENAME))
    }

    pub fn from_file<P>(path: P) -> anyhow::Result<Manifest>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let manifest = toml::from_str(
            &std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read manifest at '{}'", path.display()))?,
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

    pub fn git_deps(&self) -> impl Iterator<Item = (&String, &GitDependency)> {
        self.dependencies.iter().filter_map(|(key, val)| {
            if let Dependency::Git(git_dep) = val {
                Some((key, git_dep))
            } else {
                None
            }
        })
    }

    pub fn local_deps(&self) -> impl Iterator<Item = (&String, &LocalDependency)> {
        self.dependencies.iter().filter_map(|(key, val)| {
            if let Dependency::Local(local) = val {
                Some((key, local))
            } else {
                None
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub description: String,
    #[serde(default = "bool_true")]
    pub corlib: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Dependency {
    Simple(VersionReq),
    Advanced(AdvancedDependency),
    Git(GitDependency),
    Local(LocalDependency),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct AdvancedDependency {
    #[serde(rename = "Version")]
    pub req: VersionReq,
    #[serde(default)]
    pub features: HashSet<String>,
    #[serde(default = "bool_true")]
    pub default_features: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct GitDependency {
    pub git: url::Url,
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct LocalDependency {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Features {
    #[serde(default)]
    pub default: Vec<String>,
    #[serde(flatten)]
    pub optional: HashMap<String, Feature>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Feature {
    Project(String),
    List(Vec<String>),
}

fn bool_true() -> bool {
    true
}
