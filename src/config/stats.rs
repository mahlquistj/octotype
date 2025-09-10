use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct StatisticsConfig {
    pub save_enabled: bool,
    pub history_limit: usize,
    pub directory: Option<PathBuf>,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            save_enabled: true,
            history_limit: 10,
            directory: None,
        }
    }
}
