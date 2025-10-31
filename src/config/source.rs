use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::ParameterDefinitions;

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

pub fn get_sources(from_dir: &PathBuf) -> Result<HashMap<String, SourceConfig>, SourceError> {
    if !from_dir.exists() {
        std::fs::create_dir_all(from_dir)?;
    }

    let files = from_dir
        .read_dir()
        .map_err(|error| SourceError::ReadDirectory {
            directory: from_dir.clone(),
            error,
        })?;

    let mut sources = HashMap::new();

    for entry in files.into_iter() {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            let content = std::fs::read_to_string(path)?;
            let source: SourceConfig = toml::from_str(&content)?;
            sources.insert(source.meta.name.clone(), source);
        }
    }

    Ok(sources)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub meta: SourceMeta,
    #[serde(default)]
    pub parameters: ParameterDefinitions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub name: String,
    pub description: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub formatting: Formatting,
    #[serde(default)]
    pub network_required: bool,
    #[serde(default)]
    pub required_tools: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Formatting {
    #[default]
    Raw,
    Spaced,
}

#[cfg(test)]
mod test {
    use std::{fs::read_to_string, path::PathBuf, str::FromStr};

    use crate::config::SourceConfig;

    #[test]
    fn parse_official_sources() {
        let sources = PathBuf::from_str("./sources/").unwrap();

        for entry in sources.read_dir().unwrap().map(Result::unwrap) {
            if entry.path().extension().is_none_or(|ext| ext != "toml") {
                continue;
            };

            let source_str = read_to_string(entry.path()).unwrap();

            if let Err(error) = toml::from_str::<SourceConfig>(&source_str) {
                let name = entry.file_name();
                panic!("Failed to parse source '{name:?}': {error}",)
            }
        }
    }
}
