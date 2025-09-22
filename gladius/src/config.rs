//! # Configuration Module - Runtime Behavior Settings
//!
//! This module provides configuration options for customizing the behavior of the
//! gladius typing trainer library. Configuration affects measurement intervals,
//! performance tracking, and other runtime behaviors.
//!
//! ## Usage
//!
//! ```rust
//! use gladius::config::Configuration;
//!
//! // Use default configuration
//! let config = Configuration::default();
//!
//! // Custom configuration
//! let config = Configuration {
//!     measurement_interval_seconds: 0.5, // Take measurements every 500ms
//! };
//! ```
//!
//! ## Performance Considerations
//!
//! - **Measurement Interval**: Lower intervals provide more granular consistency analysis
//!   but increase computational overhead. Higher intervals reduce overhead but may miss
//!   short-term performance variations.

/// Runtime configuration for gladius typing analysis
///
/// Controls various aspects of how statistics are collected and processed
/// during typing sessions. All settings have sensible defaults optimized
/// for typical typing trainer usage.
///
/// # Performance Impact
///
/// Configuration choices directly affect performance:
/// - Frequent measurements enable detailed consistency analysis
/// - Less frequent measurements reduce computational overhead
/// - Default settings balance accuracy with performance
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Interval between performance measurements in seconds
    ///
    /// Controls how often WPM, IPM, accuracy, and consistency metrics are calculated
    /// and stored. Lower values provide more detailed consistency analysis but
    /// increase CPU usage.
    ///
    /// **Default**: 1.0 seconds
    /// **Range**: 0.1 - 10.0 seconds (recommended)
    /// **Impact**: Lower = better consistency tracking, higher CPU usage
    pub measurement_interval_seconds: f64,
}

impl Default for Configuration {
    /// Create configuration with recommended default values
    ///
    /// Default settings are optimized for typical typing trainer usage,
    /// balancing measurement accuracy with performance.
    ///
    /// # Default Values
    ///
    /// - `measurement_interval_seconds`: 1.0 (one measurement per second)
    fn default() -> Self {
        Self {
            measurement_interval_seconds: 1.0,
        }
    }
}
