use std::{collections::HashMap, net::TcpStream, path::PathBuf, sync::Arc, time::Duration};

use derive_more::{Deref, From};
use directories::ProjectDirs;
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use mode::ModeConfig;
pub use source::SourceConfig;

use crate::config::{stats::StatisticsConfig, theme::Theme};
use crate::statistics::{StatisticsError, StatisticsManager};

pub mod mode;
pub mod parameters;
pub mod source;
pub mod stats;
pub mod theme;

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

    #[error("Failed to initialize statistics: {0}")]
    Statistics(StatisticsError),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub theme: theme::Theme,
    pub statistic: stats::StatisticsConfig,
    sources_dir: Option<PathBuf>,
    modes_dir: Option<PathBuf>,
    pub words_per_line: usize,
    pub show_ghost_lines: usize,
    #[serde(default)]
    pub ghost_opacity: Vec<f32>,
    pub disable_ghost_fade: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            statistic: StatisticsConfig::default(),
            sources_dir: None,
            modes_dir: None,
            words_per_line: 5,
            show_ghost_lines: 3,
            ghost_opacity: get_evenly_spread_values(3),
            disable_ghost_fade: false,
        }
    }
}

fn is_online() -> bool {
    // Google's public DNS server (highly reliable)
    let address = "8.8.8.8:53";

    // Short timeout to avoid blocking the thread for too long
    let timeout = Duration::from_secs(2);

    TcpStream::connect_timeout(&address.parse().unwrap(), timeout).is_ok()
}

#[derive(Clone, Debug, Deref, Default, Serialize)]
pub struct Config(Arc<InnerConfig>);

#[derive(Debug, Default, Serialize)]
pub struct InnerConfig {
    pub settings: Settings,
    pub modes: HashMap<String, ModeConfig>,
    pub sources: HashMap<String, SourceConfig>,
    #[serde(skip)]
    pub statistics_manager: Option<StatisticsManager>,
}

impl Config {
    pub fn list_modes(&self) -> Vec<String> {
        let mut modes: Vec<_> = self.modes.keys().map(|key| key.to_string()).collect();

        modes.sort();
        modes
    }

    pub fn list_sources(&self) -> Vec<String> {
        let is_online = is_online();
        let mut sources: Vec<_> = self
            .sources
            .iter()
            .filter(|(_, cfg)| is_online || !cfg.requires_network())
            .map(|(key, _)| key.to_string())
            .collect();
        sources.sort();
        sources
    }

    pub fn sources_dir(&self) -> &PathBuf {
        self.settings.sources_dir.as_ref().unwrap()
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
        if !settings_toml.ends_with("config.toml") {
            settings_toml.push("config.toml");
        }

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

        if settings.ghost_opacity.len() != settings.show_ghost_lines {
            settings.ghost_opacity = get_evenly_spread_values(settings.show_ghost_lines);
        }

        // Initialize statistics manager if saving is enabled
        let statistics_manager = if settings.statistic.save_enabled {
            let stats_dir = settings.statistic.directory.clone().unwrap_or_else(|| {
                let mut dir = config_dir;
                dir.push("statistics");
                dir
            });
            Some(StatisticsManager::new(stats_dir)?)
        } else {
            None
        };

        Ok(Self(Arc::new(InnerConfig {
            settings,
            sources,
            modes,
            statistics_manager,
        })))
    }
}

fn get_evenly_spread_values(num_items: usize) -> Vec<f32> {
    if num_items == 0 {
        return Vec::new();
    }

    if num_items == 1 {
        return vec![0.5];
    }

    let mut values = Vec::with_capacity(num_items);
    let min_val = 0.2;
    let max_val = 0.8;
    let range: f32 = max_val - min_val;

    for i in 0..num_items {
        let normalized_index = i as f32 / (num_items - 1) as f32;
        let value = range.mul_add(normalized_index, min_val);
        values.push(value);
    }

    values
}
