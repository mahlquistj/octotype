use std::{collections::HashMap, num::ParseIntError, path::PathBuf, str::ParseBoolError};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::{ParameterDefinitions, ParameterValues, replace_parameters};

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

pub fn get_modes(from_dir: &PathBuf) -> Result<HashMap<String, ModeConfig>, ModeError> {
    if !from_dir.exists() {
        std::fs::create_dir_all(from_dir)?;
    }

    let files = from_dir
        .read_dir()
        .map_err(|error| ModeError::ReadDirectory {
            directory: from_dir.clone(),
            error,
        })?;

    let mut modes = HashMap::new();

    for entry in files.into_iter() {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            let content = std::fs::read_to_string(path)?;
            let mode: ModeConfig = toml::from_str(&content)?;
            modes.insert(mode.meta.name.clone(), mode);
        }
    }

    Ok(modes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub meta: ModeMeta,
    #[serde(default)]
    pub parameters: ParameterDefinitions,
    #[serde(default)]
    pub conditions: ConditionConfig,
    #[serde(default)]
    pub overrides: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeMeta {
    pub name: String,
    #[serde(default = "default_description")]
    pub description: String,
    pub allowed_sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    String(String),
    Number(i64),
    Bool(bool),
}

#[derive(Debug, Error)]
pub enum ParseConditionError {
    #[error("Condition '{0}' failed to parse as boolean: {0}")]
    Bool(&'static str, String),

    #[error("Condition '{0}' failed to parse as number: {0}")]
    Number(&'static str, String),
}

impl ConditionValue {
    pub fn parse_bool(
        self,
        key: &'static str,
        parameters: &ParameterValues,
    ) -> Result<bool, ParseConditionError> {
        match self {
            Self::Bool(b) => Ok(b),
            Self::String(string) => replace_parameters(&string, parameters)
                .parse::<bool>()
                .map_err(|err: ParseBoolError| ParseConditionError::Bool(key, err.to_string())),
            Self::Number(num) => Err(ParseConditionError::Bool(
                key,
                format!("Found number '{num}'"),
            )),
        }
    }

    pub fn parse_number(
        self,
        key: &'static str,
        parameters: &ParameterValues,
    ) -> Result<i64, ParseConditionError> {
        match self {
            Self::Number(num) => Ok(num),
            Self::String(string) => replace_parameters(&string, parameters)
                .parse::<i64>()
                .map_err(|err: ParseIntError| ParseConditionError::Number(key, err.to_string())),
            Self::Bool(b) => Err(ParseConditionError::Number(
                key,
                format!("Found boolean '{b}'"),
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConditionConfig {
    pub time: Option<ConditionValue>,
    pub words_typed: Option<ConditionValue>,
    pub allow_deletions: ConditionValue,
    pub allow_errors: ConditionValue,
}

impl Default for ConditionConfig {
    fn default() -> Self {
        Self {
            time: None,
            words_typed: None,
            allow_deletions: ConditionValue::Bool(true),
            allow_errors: ConditionValue::Bool(true),
        }
    }
}

pub fn default_description() -> String {
    "No description".to_string()
}
