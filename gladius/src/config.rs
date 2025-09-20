use web_time::Duration;

/// Configure how Gladius behaves
pub struct Configuration {
    /// How often Gladius should poll for WPM, IPM and Accuracy measurements
    pub measurement_interval: Duration,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            measurement_interval: Duration::from_secs(1),
        }
    }
}
