use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::{self, Definition, ParameterDefinitions};

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

    if modes.is_empty() {
        return Ok(get_default_modes());
    }

    Ok(modes)
}

pub fn get_default_modes() -> HashMap<String, ModeConfig> {
    let mut modes = HashMap::new();

    modes.insert(
        "Default".to_string(),
        ModeConfig {
            meta: ModeMeta {
                name: "Default".to_string(),
                description: "The normal typing-trainer experience".to_string(),
            },
            parameters: ParameterDefinitions::new(),
            conditions: ConditionConfig {
                allow_deletions: true,
                ..Default::default()
            },
            sources: None,
            source_overrides: HashMap::new(),
        },
    );

    let mut parameters = ParameterDefinitions::new();
    parameters.insert(
        "word_count".to_string(),
        Definition::Range {
            min: 10,
            max: usize::MAX,
            step: 10,
            default: Some(60),
        },
    );
    modes.insert(
        "WordCount".to_string(),
        ModeConfig {
            meta: ModeMeta {
                name: "WordCount".to_string(),
                description: "Type an amount of correct words".to_string(),
            },
            parameters,
            conditions: ConditionConfig {
                words_typed: Some(ConditionValue::String("{word_count}".to_string())),
                allow_deletions: false,
                ..Default::default()
            },
            sources: None,
            source_overrides: HashMap::new(),
        },
    );

    let mut parameters = ParameterDefinitions::new();
    parameters.insert(
        "time".to_string(),
        Definition::Range {
            min: 10,
            max: usize::MAX,
            step: 10,
            default: Some(60),
        },
    );
    modes.insert(
        "Timed".to_string(),
        ModeConfig {
            meta: ModeMeta {
                name: "Timed".to_string(),
                description: "A timed challenge".to_string(),
            },
            parameters,
            conditions: ConditionConfig {
                time: Some(ConditionValue::String("{time}".to_string())),
                allow_deletions: true,
                ..Default::default()
            },
            sources: None,
            source_overrides: HashMap::new(),
        },
    );

    modes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeMeta {
    pub name: String,
    #[serde(default = "default_description")]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionValue {
    String(String),
    Number(usize),
    Bool(bool),
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ConditionConfig {
    pub time: Option<ConditionValue>,
    pub words_typed: Option<ConditionValue>,
    #[serde(default = "default_allow_deletions")]
    pub allow_deletions: bool,
}

pub fn default_description() -> String {
    "No description".to_string()
}

pub const fn default_allow_deletions() -> bool {
    true
}
