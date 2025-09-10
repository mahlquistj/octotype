use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ParameterValues;

#[derive(Debug, From, Error)]
pub enum SourceError {
    #[error("Failed to read sources directory '{directory}': {error}")]
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

pub fn get_sources(from_dir: PathBuf) -> Result<HashMap<String, SourceConfig>, SourceError> {
    if !from_dir.exists() {
        std::fs::create_dir_all(&from_dir)?;
    }

    let files = from_dir
        .read_dir()
        .map_err(|error| SourceError::ReadDirectory {
            directory: from_dir,
            error,
        })?;

    let mut modes = HashMap::new();

    for entry in files.into_iter() {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if path.is_file() {
            let content = std::fs::read_to_string(path)?;
            let mode: SourceConfig = toml::from_str(&content)?;
            modes.insert(mode.meta.name.clone(), mode);
        }
    }

    Ok(modes)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfig {
    pub meta: SourceMeta,
    pub parameters: ParameterValues,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceMeta {
    name: String,
    description: String,
    command: Vec<String>,
    output: OutputFormat,
    #[serde(default)]
    offline_alternative: Option<String>,
    #[serde(default)]
    network_required: bool,
    #[serde(default)]
    required_tools: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Default,
    Array,
}
