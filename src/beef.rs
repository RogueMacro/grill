use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BeefSpace {
    pub file_version: u32,
    #[serde(default)]
    pub locked: HashSet<String>,
    #[serde(default)]
    pub projects: HashMap<String, ProjectEntry>,
    #[serde(default)]
    pub workspace_folders: HashMap<String, HashSet<String>>,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

impl Default for BeefSpace {
    fn default() -> Self {
        Self {
            file_version: 1,
            locked: Default::default(),
            projects: Default::default(),
            workspace_folders: Default::default(),
            other: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ProjectEntry {
    pub path: PathBuf,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

impl Default for ProjectEntry {
    fn default() -> Self {
        Self {
            path: Default::default(),
            other: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BeefProj {
    pub file_version: u32,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    pub project: Project,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    pub name: String,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}
