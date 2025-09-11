use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::{Definition, ParameterDefinitions};

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

    let mut sources = HashMap::new();

    for entry in files.into_iter() {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if path.is_file() {
            let content = std::fs::read_to_string(path)?;
            let source: SourceConfig = toml::from_str(&content)?;
            sources.insert(source.meta.name.clone(), source);
        }
    }

    if sources.is_empty() {
        return Ok(get_default_sources());
    }

    Ok(sources)
}

pub fn get_default_sources() -> HashMap<String, SourceConfig> {
    let mut sources = HashMap::new();

    let mut parameters = ParameterDefinitions::new();
    parameters.insert(
        "word_count".to_string(),
        Definition::Range {
            min: 1,
            max: 30,
            step: 1,
            default: Some(10),
        },
    );
    parameters.insert(
        "word_length".to_string(),
        Definition::Range {
            min: 2,
            max: 15,
            step: 1,
            default: Some(5),
        },
    );
    sources.insert(
        "Gibberish".to_string(),
        SourceConfig {
            meta: SourceMeta {
                name: "Quotes API".to_string(),
                description: "Supplies random quotes".to_string(),
                command: [
                    "tr",
                    "-dc",
                    "'a-zA-Z0-9'",
                    "<",
                    "/dev/urandom",
                    "|",
                    "head",
                    "-c",
                    "$(({word_count} * {word_length}))",
                    "|",
                    "fold",
                    "-w",
                    "{word_length}",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
                required_tools: vec!["tr".to_string(), "head".to_string(), "fold".to_string()],
                output: OutputFormat::Default,
                offline_alternative: None,
                network_required: false,
            },
            parameters,
        },
    );

    sources.insert(
        "pwd".to_string(),
        SourceConfig {
            meta: SourceMeta {
                name: "Quotes API".to_string(),
                description: "Supplies random quotes".to_string(),
                command: vec!["echo".to_string(), "$(pwd)".to_string()],
                required_tools: vec!["tr".to_string(), "head".to_string(), "fold".to_string()],
                output: OutputFormat::Default,
                offline_alternative: None,
                network_required: false,
            },
            parameters: ParameterDefinitions::new(),
        },
    );

    sources
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub meta: SourceMeta,
    pub parameters: ParameterDefinitions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub name: String,
    pub description: String,
    pub command: Vec<String>,
    pub output: OutputFormat,
    #[serde(default)]
    pub offline_alternative: Option<String>,
    #[serde(default)]
    pub network_required: bool,
    #[serde(default)]
    pub required_tools: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Default,
    Array,
}
