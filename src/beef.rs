use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
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
    pub workspace: Workspace,

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
            workspace: Default::default(),
            other: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ProjectEntry {
    pub path: PathBuf,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Workspace {
    pub startup_project: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BeefProj {
    pub file_version: u32,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    pub project: Project,

    #[serde(flatten)]
    other: HashMap<String, toml::Value>,

    #[serde(skip)]
    path: PathBuf,
}

impl BeefProj {
    pub fn new<P>(name: String, path: &P) -> BeefProj
    where
        P: AsRef<Path>,
    {
        let startup_object = format!("{}.Program", name);

        Self {
            file_version: 1,
            dependencies: Default::default(),
            project: Project {
                name,
                target_type: String::from("BeefConsoleApplication"),
                startup_object,
                ..Default::default()
            },
            other: Default::default(),
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn from_file<P>(path: &P) -> Result<BeefProj>
    where
        P: AsRef<Path>,
    {
        let mut proj: Self = toml::from_str(&fs::read_to_string(&path)?).with_context(|| {
            format!("Failed to read project file '{}'", path.as_ref().display())
        })?;
        proj.path = path.as_ref().to_path_buf();
        Ok(proj)
    }

    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, toml::to_string(&self)?)
            .with_context(|| format!("Failed to write project file: '{}'", self.path.display()))
    }

    pub fn path<P>(&mut self, path: &P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.path = path.as_ref().to_path_buf();
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub target_type: String,
    #[serde(default)]
    pub startup_object: String,
    #[serde(default)]
    pub processor_macros: HashSet<String>,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}
