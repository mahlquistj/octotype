/// Configure how Gladius behaves
#[derive(Debug, Clone)]
pub struct Configuration {
    /// How often Gladius should poll for WPM, IPM and Accuracy measurements
    pub measurement_interval_seconds: f64,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            measurement_interval_seconds: 1.0,
        }
    }
}
