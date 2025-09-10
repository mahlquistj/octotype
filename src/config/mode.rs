use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::{Definition, ParameterDefinitions};

#[derive(Debug, From, Error)]
pub enum ModeError {
    #[error("Failed to read modes directory '{directory}': {error}")]
    #[from(skip)]
    ReadDirectory {
        directory: PathBuf,
        error: std::io::Error,
    },

    #[error("Failed to read file")]
    ReadFile(std::io::Error),

    #[error("Failed to parse file")]
    ParseFile(toml::de::Error),
}

pub fn get_modes(from_dir: PathBuf) -> Result<HashMap<String, ModeConfig>, ModeError> {
    if !from_dir.exists() {
        std::fs::create_dir_all(&from_dir)?;
    }

    let files = from_dir
        .read_dir()
        .map_err(|error| ModeError::ReadDirectory {
            directory: from_dir,
            error,
        })?;

    let mut modes = HashMap::new();

    for entry in files.into_iter() {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if path.is_file() {
            let content = std::fs::read_to_string(path)?;
            let mode: ModeConfig = toml::from_str(&content)?;
            modes.insert(mode.meta.name.clone(), mode);
        }
    }

    Ok(modes)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModeConfig {
    pub meta: ModeMeta,
    #[serde(default)]
    pub parameters: ParameterDefinitions,
    #[serde(default)]
    pub conditions: ConditionConfig,
    #[serde(default)]
    pub source_overrides: HashMap<String, HashMap<String, String>>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModeMeta {
    pub name: String,
    #[serde(default = "default_description")]
    pub description: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ConditionConfig {
    pub time: Option<Definition>,
    pub words_typed: Option<Definition>,
    #[serde(default)]
    pub allow_deletions: bool,
}

pub fn default_description() -> String {
    "No description".to_string()
}
