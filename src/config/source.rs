use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::parameters::ParameterDefinitions;

const BROWNFOX_TEXT: &str = "The quick brown fox jumps over the lazy dog, testing my typing speed with every leap, but I'll soon catch up.";
const NUMBER_WORDS: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety", "hundred",
];

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

    #[error("Failed to serialize default source")]
    SerializeDefault(toml::ser::Error),
}

pub fn create_default_sources() -> HashMap<String, SourceConfig> {
    let mut sources = HashMap::new();
    sources.insert(
        "brownfox".to_string(),
        SourceConfig {
            meta: SourceMeta {
                name: "BrownFox".to_string(),
                description: "The quick brown fox...".to_string(),
            },
            generator: GeneratorDefinition::List {
                source: ListSource::Array(
                    BROWNFOX_TEXT
                        .split_ascii_whitespace()
                        .map(str::to_string)
                        .collect(),
                ),
                randomize: false,
            },
            parameters: HashMap::new(),
        },
    );
    sources.insert(
        "number_words".to_string(),
        SourceConfig {
            meta: SourceMeta {
                name: "NumberWords".to_string(),
                description: "Numbers as words (one, two, three...)".to_string(),
            },
            generator: GeneratorDefinition::List {
                source: ListSource::Array(NUMBER_WORDS.into_iter().map(str::to_string).collect()),
                randomize: true,
            },
            parameters: HashMap::new(),
        },
    );

    sources
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

    if sources.is_empty() {
        let sources = create_default_sources();
        for (filename, source) in sources {
            let mut location = from_dir.clone();
            location.push(filename);
            location.set_extension("toml");

            File::create(location)?.write_all(toml::to_string_pretty(source)?)
        }
    }

    Ok(sources)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub meta: SourceMeta,
    #[serde(default)]
    pub parameters: ParameterDefinitions,
    pub generator: GeneratorDefinition,
}

impl SourceConfig {
    pub const fn requires_network(&self) -> bool {
        if let GeneratorDefinition::Command {
            network_required, ..
        } = self.generator
        {
            network_required
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratorDefinition {
    Command {
        command: Vec<String>,
        #[serde(default)]
        formatting: Formatting,
        #[serde(default)]
        network_required: bool,
        #[serde(default)]
        required_tools: Vec<String>,
    },
    List {
        source: ListSource,
        randomize: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListSource {
    Array(Vec<String>),
    File {
        path: PathBuf,
        seperator: Option<char>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub name: String,
    pub description: String,
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
