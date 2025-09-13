use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use directories::ProjectDirs;
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use mode::ModeConfig;
pub use source::SourceConfig;

pub mod mode;
pub mod parameters;
pub mod source;
pub mod stats;
pub mod theme;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub theme: theme::Theme,
    pub statistic: stats::StatisticsConfig,
    pub sources_dir: Option<PathBuf>,
    pub modes_dir: Option<PathBuf>,
}

#[derive(Debug, From, Error)]
pub enum ConfigError {
    #[error(
        "Failed to get configuration directory. Please specify the location using the `--config <path>` flag"
    )]
    NoDirectory,

    #[error("Failed to create config directory: {0}")]
    CreateDirectory(std::io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(Box<figment::Error>),

    #[error("Failed to parse sources: {0}")]
    ParseSources(source::SourceError),

    #[error("Failed to parse modes: {0}")]
    ParseModes(mode::ModeError),
}

#[derive(Debug, Default)]
pub struct Config {
    pub settings: Settings,
    pub modes: HashMap<String, ModeConfig>,
    pub sources: HashMap<String, SourceConfig>,
}

impl Config {
    pub fn list_modes(&self) -> Vec<String> {
        self.modes.keys().map(|key| key.to_string()).collect()
    }

    pub fn list_sources(&self) -> Vec<String> {
        self.sources.keys().map(|key| key.to_string()).collect()
    }

    pub fn get(override_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        // Grab default configuration
        let mut settings = Figment::from(Serialized::defaults(Settings::default()));

        // Check for toml file location
        let config_dir = override_path
            .or_else(|| {
                ProjectDirs::from("com", "OctoType", "OctoType")
                    .map(|dirs| dirs.config_dir().to_path_buf())
            })
            .ok_or(ConfigError::NoDirectory)?;

        // Ensure path exists
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let mut settings_toml = config_dir.clone();
        settings_toml.push("settings.toml");

        if settings_toml.exists() {
            settings = settings.merge(Toml::file(settings_toml));
        }

        let mut settings: Settings = settings.extract().map_err(Box::new)?;

        let sources_dir = settings.sources_dir.clone().unwrap_or_else(|| {
            let mut dir = config_dir.clone();
            dir.push("sources");
            dir
        });
        let sources = source::get_sources(&sources_dir)?;
        settings.sources_dir = Some(sources_dir);

        let modes_dir = settings.modes_dir.clone().unwrap_or_else(|| {
            let mut dir = config_dir.clone();
            dir.push("modes");
            dir
        });
        let modes = mode::get_modes(&modes_dir)?;
        settings.modes_dir = Some(modes_dir);

        Ok(Self {
            settings,
            sources,
            modes,
        })
    }
}
