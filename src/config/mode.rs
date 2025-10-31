use std::{collections::HashMap, num::ParseIntError, path::PathBuf, str::ParseBoolError};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::{self, ParameterDefinitions, ParameterValues};

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

pub fn create_default_modes() -> HashMap<String, ModeConfig> {
    let mut modes = HashMap::new();
    modes.insert(
        "Default".to_string(),
        ModeConfig {
            meta: ModeMeta {
                name: "Default".to_string(),
                description: "The default typing-trainer experience".to_string(),
                allowed_sources: None,
            },
            parameters: HashMap::new(),
            conditions: ConditionConfig::default(),
            overrides: HashMap::new(),
        },
    );
    modes.insert(
        "WordRace".to_string(),
        ModeConfig {
            meta: ModeMeta {
                name: "WordRace".to_string(),
                description: "Type an amount of correct words within the time limit".to_string(),
                allowed_sources: None,
            },
            parameters: [
                (
                    "words".to_string(),
                    parameters::Definition::Range {
                        min: 10,
                        max: i64::MAX,
                        step: 2,
                        default: Some(30),
                        value: 30,
                    },
                ),
                (
                    "time (seconds)".to_string(),
                    parameters::Definition::Range {
                        min: 10,
                        max: i64::MAX,
                        step: 5,
                        default: Some(60),
                        value: 60,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            conditions: ConditionConfig {
                words_typed: Some(ConditionValue::String("{words}".to_string())),
                ..Default::default()
            },
            overrides: HashMap::new(),
        },
    );
    modes.insert(
        "Perfectionism".to_string(),
        ModeConfig {
            meta: ModeMeta {
                name: "Perfectionism".to_string(),
                description: "Don't make any mistakes!".to_string(),
                allowed_sources: None,
            },
            parameters: HashMap::new(),
            conditions: ConditionConfig {
                allow_errors: ConditionValue::Bool(false),
                ..Default::default()
            },
            overrides: HashMap::new(),
        },
    );

    modes
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
    #[serde(default = "ModeMeta::default_description")]
    pub description: String,
    pub allowed_sources: Option<Vec<String>>,
}

impl ModeMeta {
    fn default_description() -> String {
        "No description".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    String(String),
    Number(usize),
    Bool(bool),
}

#[derive(Debug, Error)]
pub enum ParseConditionError {
    #[error("Condition '{0}' failed to parse as boolean: {1}")]
    Bool(&'static str, String),

    #[error("Condition '{0}' failed to parse as number: {1}")]
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
            Self::String(string) => parameters
                .replace_values(&string)
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
    ) -> Result<usize, ParseConditionError> {
        match self {
            Self::Number(num) => Ok(num),
            Self::String(string) => parameters
                .replace_values(&string)
                .parse::<usize>()
                .map_err(|err: ParseIntError| ParseConditionError::Number(key, err.to_string())),
            Self::Bool(b) => Err(ParseConditionError::Number(
                key,
                format!("Found boolean '{b}'"),
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
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

#[cfg(test)]
mod test {
    use std::{fs::read_to_string, path::PathBuf, str::FromStr};

    use crate::config::ModeConfig;

    #[test]
    fn parse_official_modes() {
        let modes = PathBuf::from_str("./modes/").unwrap();

        for entry in modes.read_dir().unwrap().map(Result::unwrap) {
            if entry.path().extension().is_none_or(|ext| ext != "toml") {
                continue;
            };

            let mode_str = read_to_string(entry.path()).unwrap();

            if let Err(error) = toml::from_str::<ModeConfig>(&mode_str) {
                let name = entry.file_name();
                panic!("Failed to parse mode '{name:?}': {error}",)
            }
        }
    }
}
