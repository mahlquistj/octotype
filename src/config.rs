use serde::{Deserialize, Serialize};

mod theme;

use theme::Theme;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub theme: Theme,
    pub statistic: StatisticsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatisticsConfig {
    pub save_enabled: bool,
    pub history_limit: usize,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            save_enabled: true,
            history_limit: 10,
        }
    }
}
