//! # Math Module - Typing Performance Calculations
//!
//! This module provides mathematical functions for calculating various typing performance metrics
//! including Words Per Minute (WPM), Inputs Per Minute (IPM), Accuracy, and Consistency.
//!
//! ## Mathematical Foundations
//!
//! The calculations in this module are based on standard typing performance metrics used in
//! typing trainers and research.
//!
//! ## Key Concepts
//!
//! - **Error**: A keystroke that doesn't match the expected character
//! - **Correction**: A keystroke that fixes a previously made error
//! - **Input**: Any keystroke including additions, deletions, and corrections

use crate::{Float, Minutes};

/// The average word length in the english dictionary (industry standard for typing trainers)
///
/// Used to calculate [Wpm]
pub const AVERAGE_WORD_LENGTH: usize = 5;

/// # Words Per Minute (WPM)
///
/// Measures typing speed by calculating how many words (assuming 5 characters per word)
/// are typed per minute. This is the most common metric for typing speed assessment.
///
/// ## Mathematical Formulas
///
/// ### Raw WPM
///
/// $$WPM_{raw} = \frac{C}{L \cdot T}$$
///
/// Where:
/// - $C$ = total characters typed
/// - $L$ = [AVERAGE_WORD_LENGTH]
/// - $T$ = time in minutes
///
/// ### Corrected WPM
///
/// $$WPM_{corrected} = WPM_{raw} - \frac{E}{T}$$
///
/// Where:
/// - $E$ = total errors made
///
/// ### Actual WPM
///
/// $$WPM_{actual} = WPM_{raw} - \frac{E + R}{T}$$
///
/// Where:
/// - $R$ = total corrections made
///
/// ## Usage Notes
///
/// - Raw WPM shows pure typing speed without quality consideration
/// - Corrected WPM penalizes errors but rewards fixing them
/// - Actual WPM penalizes both errors and the time spent correcting them
/// - Negative values are possible if error rates are extremely high
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Wpm {
    /// Raw WPM: Pure typing speed without error consideration
    ///
    /// Formula: `(characters / 5) / minutes`
    pub raw: Float,

    /// Corrected WPM: Raw WPM minus errors per minute
    ///
    /// Formula: `raw_wpm - (errors / minutes)`
    pub corrected: Float,

    /// Actual WPM: Raw WPM minus both errors and corrections per minute
    ///
    /// Formula: `raw_wpm - ((errors + corrections) / minutes)`
    pub actual: Float,
}

impl Wpm {
    /// Calculate Words Per Minute using the formulas described above
    ///
    /// # Parameters
    ///
    /// * `characters` - Total number of characters typed during the session
    /// * `errors` - Total number of errors made during the session  
    /// * `corrections` - Total number of corrections made during the session
    /// * `minutes` - Duration of the typing session in minutes
    ///
    /// # Returns
    ///
    /// A `Wpm` struct containing raw, corrected, and actual WPM calculations
    ///
    /// # Example
    ///
    /// ```
    /// use gladius::math::Wpm;
    ///
    /// let wpm = Wpm::calculate(250, 5, 2, 5.0);
    /// println!("Raw WPM: {}", wpm.raw);     // 10.0 WPM
    /// println!("Corrected: {}", wpm.corrected); // 9.0 WPM  
    /// println!("Actual: {}", wpm.actual);   // 8.6 WPM
    /// ```
    pub fn calculate(
        characters: usize,
        errors: usize,
        corrections: usize,
        minutes: Minutes,
    ) -> Self {
        let characters = characters as Float;
        let errors = errors as Float;
        let corrections = corrections as Float;

        // Errors Per Minute
        let epm = errors / minutes;
        // Corrections and Errors Per Minute
        let cepm = (corrections + errors) / minutes;

        // Raw WPM
        let raw = (characters / AVERAGE_WORD_LENGTH as Float) / minutes;

        // Corrected WPM
        let corrected = raw - epm;

        // Actual WPM
        let actual = raw - cepm;

        Self {
            raw,
            corrected,
            actual,
        }
    }
}

/// # Inputs Per Minute (IPM)
///
/// Measures the raw input speed by counting all keystrokes, including corrections and deletions.
/// This metric provides insight into actual typing activity versus productive character output.
///
/// ## Mathematical Formulas
///
/// ### Raw IPM
///
/// $$IPM_{raw} = \frac{I_{total}}{T}$$
///
/// Where:
/// - $I_{total}$ = total number of keystrokes (including deletions, corrections)
/// - $T$ = time in minutes
///
/// ### Actual IPM
///
/// $$IPM_{actual} = \frac{I_{productive}}{T}$$
///
/// Where:
/// - $I_{productive}$ = number of keystrokes that added characters to the input
///
/// ## Usage Notes
///
/// - Raw IPM shows total keyboard activity including corrections
/// - Actual IPM shows productive keystroke rate
/// - Higher ratios of actual/raw indicate more accurate typing
/// - Useful for identifying excessive correction patterns
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ipm {
    /// Raw IPM: Total keystrokes per minute including deletions and corrections
    ///
    /// Formula: $\frac{\text{total keystrokes}}{\text{minutes}}$
    pub raw: Float,

    /// Actual IPM: Productive keystrokes per minute (characters added to input)
    ///
    /// Formula: $\frac{\text{productive keystrokes}}{\text{minutes}}$
    pub actual: Float,
}

impl Ipm {
    /// Calculate Inputs Per Minute using the formulas described above
    ///
    /// # Parameters
    ///
    /// * `actual_inputs` - Number of productive keystrokes (characters added to input)
    /// * `raw_inputs` - Total number of keystrokes including deletions and corrections
    /// * `minutes` - Duration of the typing session in minutes
    ///
    /// # Returns
    ///
    /// An `Ipm` struct containing raw and actual IPM calculations
    ///
    /// # Example
    ///
    /// ```
    /// use gladius::math::Ipm;
    ///
    /// let ipm = Ipm::calculate(240, 300, 5.0);
    /// println!("Raw IPM: {}", ipm.raw);     // 60.0 IPM
    /// println!("Actual IPM: {}", ipm.actual); // 48.0 IPM
    /// // Efficiency: 48/60 = 80%
    /// ```
    pub fn calculate(actual_inputs: usize, raw_inputs: usize, minutes: Minutes) -> Self {
        let raw_inputs = raw_inputs as Float;
        let actual_inputs = actual_inputs as Float;

        Self {
            raw: raw_inputs / minutes,
            actual: actual_inputs / minutes,
        }
    }
}

/// # Typing Accuracy
///
/// Measures typing precision as the percentage of correctly typed characters.
/// Provides both raw accuracy (counting corrections as valid) and actual accuracy
/// (penalizing corrections as inefficiency).
///
/// ## Mathematical Formulas
///
/// ### Raw Accuracy
///
/// $$A_{raw} = \left(1 - \frac{E}{L}\right) \times 100\%$$
///
/// Where:
/// - $E$ = total errors made
/// - $L$ = total input length (characters typed)
///
/// ### Actual Accuracy
///
/// $$A_{actual} = \left(1 - \frac{\max(0, E - R)}{L}\right) \times 100\%$$
///
/// Where:
/// - $R$ = total corrections made
/// - $\max(0, E - R)$ ensures non-negative error count
///
/// ## Usage Notes
///
/// - Raw accuracy treats corrected errors as if they never happened
/// - Actual accuracy only counts corrections if they exceed total errors
/// - Values range from 0.0% (all errors) to 100.0% (perfect typing)
/// - Actual accuracy can be higher than raw when corrections > errors
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Accuracy {
    /// Raw accuracy: Percentage treating corrections as valid characters
    ///
    /// Formula: $(1 - \frac{\text{errors}}{\text{input length}}) \times 100\%$
    pub raw: Float,

    /// Actual accuracy: Percentage considering net errors after corrections
    ///
    /// Formula: $(1 - \frac{\max(0, \text{errors} - \text{corrections})}{\text{input length}}) \times 100\%$
    pub actual: Float,
}

impl Accuracy {
    /// Calculate typing accuracy using the formulas described above
    ///
    /// # Parameters
    ///
    /// * `input_len` - Total length of the input text (characters typed)
    /// * `total_errors` - Total number of errors made during typing
    /// * `total_corrections` - Total number of corrections made during typing
    ///
    /// # Returns
    ///
    /// An `Accuracy` struct containing raw and actual accuracy percentages
    ///
    /// # Example
    ///
    /// ```
    /// use gladius::math::Accuracy;
    ///
    /// let accuracy = Accuracy::calculate(100, 8, 5);
    /// println!("Raw accuracy: {}%", accuracy.raw);    // 92.0%
    /// println!("Actual accuracy: {}%", accuracy.actual); // 97.0%
    /// // Net errors: max(0, 8-5) = 3 errors
    /// ```
    pub fn calculate(input_len: usize, total_errors: usize, total_corrections: usize) -> Self {
        let input_len = input_len as Float;
        let total_errors = total_errors as Float;
        let total_corrections = total_corrections as Float;
        let actual_errors = (total_errors - total_corrections).max(0.0);

        Self {
            raw: (1.0 - (total_errors / input_len)) * 100.0,
            actual: (1.0 - (actual_errors / input_len)) * 100.0,
        }
    }
}

/// # Typing Consistency
///
/// Measures the stability and regularity of typing speed over time using statistical analysis
/// of WPM measurements. Consistency is calculated using the coefficient of variation (CV)
/// and converted to a percentage where higher values indicate more consistent typing.
///
/// ## Mathematical Formulas
///
/// ### Standard Deviation (Welford's Algorithm)
///
/// For numerically stable variance calculation:
///
/// $$\sigma = \sqrt{\frac{M_2}{n}}$$
///
/// Where $M_2$ is computed using Welford's online algorithm:
///
/// $$\delta = x_i - \mu_{i-1}$$
///
/// $$\mu_i = \mu_{i-1} + \frac{\delta}{i}$$
///
/// $$\delta_2 = x_i - \mu_i$$
///
/// $$M_{2,i} = M_{2,i-1} + \delta \cdot \delta_2$$
///
/// ### Coefficient of Variation
///
/// $$CV = \frac{\sigma}{\mu}$$
///
/// Where:
/// - $\sigma$ = standard deviation of WPM measurements
/// - $\mu$ = mean of WPM measurements
///
/// ### Consistency Percentage
///
/// $$C = \max(0, (1 - \min(1, CV)) \times 100\%)$$
///
/// ## Usage Notes
///
/// - Uses population standard deviation (not sample)
/// - Welford's algorithm provides numerical stability for large datasets
/// - CV normalizes consistency relative to typing speed
/// - Perfect consistency (identical speeds) = 100%
/// - High variation (CV ≥ 1.0) = 0% consistency
/// - Expert typists typically show >80% consistency
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Consistency {
    /// Raw WPM standard deviation using Welford's algorithm
    ///
    /// Formula: $\sigma_{raw} = \sqrt{\frac{M_2}{n}}$
    pub raw_deviation: Float,

    /// Raw consistency percentage (0.0 - 100.0)
    ///
    /// Formula: $\max(0, (1 - \min(1, \frac{\sigma_{raw}}{\mu_{raw}})) \times 100\%)$
    pub raw_percent: Float,

    /// Corrected WPM standard deviation
    ///
    /// Formula: $\sigma_{corrected} = \sqrt{\frac{M_2}{n}}$
    pub corrected_deviation: Float,

    /// Corrected consistency percentage (0.0 - 100.0)
    ///
    /// Formula: $\max(0, (1 - \min(1, \frac{\sigma_{corrected}}{\mu_{corrected}})) \times 100\%)$
    pub corrected_percent: Float,

    /// Actual WPM standard deviation
    ///
    /// Formula: $\sigma_{actual} = \sqrt{\frac{M_2}{n}}$
    pub actual_deviation: Float,

    /// Actual consistency percentage (0.0 - 100.0)
    ///
    /// Formula: $\max(0, (1 - \min(1, \frac{\sigma_{actual}}{\mu_{actual}})) \times 100\%)$
    pub actual_percent: Float,
}

impl Consistency {
    /// Calculate typing consistency using the formulas described above
    ///
    /// Analyzes WPM measurements over time to determine typing consistency using
    /// Welford's algorithm for numerical stability and coefficient of variation
    /// for normalization.
    ///
    /// # Parameters
    ///
    /// * `measurements` - Slice of WPM measurements collected during typing session
    ///
    /// # Returns
    ///
    /// A `Consistency` struct containing standard deviations and percentage consistency
    /// for raw, corrected, and actual WPM measurements
    ///
    /// # Example
    ///
    /// ```
    /// use gladius::math::{Wpm, Consistency};
    ///
    /// let measurements = vec![
    ///     Wpm { raw: 50.0, corrected: 48.0, actual: 46.0 },
    ///     Wpm { raw: 52.0, corrected: 50.0, actual: 48.0 },
    ///     Wpm { raw: 49.0, corrected: 47.0, actual: 45.0 },
    /// ];
    ///
    /// let consistency = Consistency::calculate(&measurements);
    /// println!("Raw consistency: {}%", consistency.raw_percent);
    /// println!("Standard deviation: {}", consistency.raw_deviation);
    /// ```
    ///
    /// # Edge Cases
    ///
    /// - Single measurement: Returns 0 deviation, 100% consistency
    /// - Empty slice: Returns 0 deviation, 100% consistency  
    /// - Zero mean: Returns 100% consistency (prevents division by zero)
    /// - High CV (≥1.0): Returns 0% consistency
    pub fn calculate(measurements: &[Wpm]) -> Self {
        let raw_wpm_values: Vec<Float> = measurements.iter().map(|m| m.raw).collect();
        let corrected_wpm_values: Vec<Float> = measurements.iter().map(|m| m.corrected).collect();
        let actual_wpm_values: Vec<Float> = measurements.iter().map(|m| m.actual).collect();
        let raw_deviation = Self::calculate_std_dev(&raw_wpm_values);
        let corrected_deviation = Self::calculate_std_dev(&corrected_wpm_values);
        let actual_deviation = Self::calculate_std_dev(&actual_wpm_values);

        Self {
            raw_deviation,
            raw_percent: Self::cv_to_percentage(
                raw_deviation,
                Self::calculate_mean(&raw_wpm_values),
            ),
            corrected_deviation,
            corrected_percent: Self::cv_to_percentage(
                corrected_deviation,
                Self::calculate_mean(&corrected_wpm_values),
            ),
            actual_deviation,
            actual_percent: Self::cv_to_percentage(
                actual_deviation,
                Self::calculate_mean(&actual_wpm_values),
            ),
        }
    }

    /// Calculate standard deviation using Welford's online algorithm
    ///
    /// This implementation provides numerical stability for large datasets and
    /// avoids potential overflow issues with the naive two-pass algorithm.
    ///
    /// # Algorithm
    ///
    /// Implements the formulas:
    /// - $\delta = x_i - \mu_{i-1}$
    /// - $\mu_i = \mu_{i-1} + \frac{\delta}{i}$  
    /// - $\delta_2 = x_i - \mu_i$
    /// - $M_{2,i} = M_{2,i-1} + \delta \cdot \delta_2$
    /// - $\sigma = \sqrt{\frac{M_2}{n}}$
    ///
    /// # Parameters
    ///
    /// * `values` - Slice of floating point values
    ///
    /// # Returns
    ///
    /// Population standard deviation, or 0.0 for single/empty datasets
    fn calculate_std_dev(values: &[Float]) -> Float {
        if values.len() <= 1 {
            return 0.0;
        }

        // Welford's online algorithm for numerically stable variance calculation
        let mut mean = 0.0;
        let mut m2 = 0.0; // Sum of squares of deviations from mean (M₂)

        for (i, &value) in values.iter().enumerate() {
            let delta = value - mean; // δ = xᵢ - x̄ᵢ₋₁
            mean += delta / (i + 1) as Float; // x̄ᵢ = x̄ᵢ₋₁ + δ/i
            let delta2 = value - mean; // δ₂ = xᵢ - x̄ᵢ
            m2 += delta * delta2; // M₂ᵢ = M₂ᵢ₋₁ + δ·δ₂
        }

        // Population standard deviation: σ = √(M₂/n)
        let variance = m2 / values.len() as Float;
        variance.sqrt()
    }

    /// Calculate arithmetic mean of a slice of values
    ///
    /// # Formula
    ///
    /// $$\mu = \frac{1}{n}\sum_{i=1}^{n} x_i$$
    ///
    /// # Parameters
    ///
    /// * `values` - Slice of floating point values
    ///
    /// # Returns
    ///
    /// Arithmetic mean, or 0.0 for empty slice
    fn calculate_mean(values: &[Float]) -> Float {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<Float>() / values.len() as Float
        }
    }

    /// Convert coefficient of variation to consistency percentage
    ///
    /// # Formula
    ///
    /// $$C = \max(0, (1 - \min(1, \frac{\sigma}{\mu})) \times 100\%)$$
    ///
    /// # Parameters
    ///
    /// * `std_dev` - Standard deviation of the measurements
    /// * `mean` - Mean of the measurements
    ///
    /// # Returns
    ///
    /// Consistency percentage (0.0 - 100.0):
    /// - 100.0% = Perfect consistency (CV = 0)
    /// - 0.0% = High variation (CV ≥ 1.0)  
    /// - Special case: Returns 100.0% when mean is 0 (no typing activity)
    fn cv_to_percentage(std_dev: Float, mean: Float) -> Float {
        if mean == 0.0 {
            return 100.0; // Perfect consistency if no typing occurred
        }
        let cv = std_dev / mean; // Coefficient of variation
        let consistency_percent = (1.0 - cv.min(1.0)) * 100.0;
        consistency_percent.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_calculations() {
        // Test basic WPM calculation: 50 chars, 0 errors, 0 corrections, 1 minute
        // Expected: 50 / 5 = 10 WPM (assuming AVERAGE_WORD_LENGTH is 5)
        let wpm = Wpm::calculate(50, 0, 0, 1.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 10.0);
        assert_eq!(wpm.actual, 10.0);

        // Test with errors: 50 chars, 2 errors, 0 corrections, 1 minute
        let wpm = Wpm::calculate(50, 2, 0, 1.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 8.0); // 10 - 2 errors
        assert_eq!(wpm.actual, 8.0); // 10 - (2 errors + 0 corrections)

        // Test with errors and corrections: 50 chars, 3 errors, 1 correction, 1 minute
        let wpm = Wpm::calculate(50, 3, 1, 1.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 7.0); // 10 - 3 errors
        assert_eq!(wpm.actual, 6.0); // 10 - (3 errors + 1 correction)

        // Test with different time: 100 chars, 0 errors, 0 corrections, 2 minutes
        let wpm = Wpm::calculate(100, 0, 0, 2.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 10.0);
        assert_eq!(wpm.actual, 10.0);
    }

    #[test]
    fn test_ipm_calculations() {
        // Test basic IPM: 60 actual inputs, 80 raw inputs, 1 minute
        let ipm = Ipm::calculate(60, 80, 1.0);
        assert_eq!(ipm.actual, 60.0);
        assert_eq!(ipm.raw, 80.0);

        // Test with different time: 120 actual inputs, 150 raw inputs, 2 minutes
        let ipm = Ipm::calculate(120, 150, 2.0);
        assert_eq!(ipm.actual, 60.0);
        assert_eq!(ipm.raw, 75.0);

        // Test where actual equals raw (no deletions/retyping)
        let ipm = Ipm::calculate(50, 50, 1.0);
        assert_eq!(ipm.actual, 50.0);
        assert_eq!(ipm.raw, 50.0);
    }

    #[test]
    fn test_accuracy_calculations() {
        // Test perfect accuracy: 100 chars, 0 errors, 0 corrections
        let accuracy = Accuracy::calculate(100, 0, 0);
        assert_eq!(accuracy.raw, 100.0);
        assert_eq!(accuracy.actual, 100.0);

        // Test with errors but no corrections: 100 chars, 5 errors, 0 corrections
        let accuracy = Accuracy::calculate(100, 5, 0);
        assert_eq!(accuracy.raw, 95.0);
        assert_eq!(accuracy.actual, 95.0);

        // Test with errors and corrections: 100 chars, 10 errors, 6 corrections
        let accuracy = Accuracy::calculate(100, 10, 6);
        assert_eq!(accuracy.raw, 90.0); // (1 - 10/100) * 100
        assert_eq!(accuracy.actual, 96.0); // (1 - (10-6)/100) * 100

        // Test edge case: more corrections than errors
        let accuracy = Accuracy::calculate(100, 5, 8);
        assert_eq!(accuracy.raw, 95.0);
        assert_eq!(accuracy.actual, 100.0); // Clamped to 0 errors
    }

    #[test]
    fn test_fractional_time() {
        // Test with 30 seconds (0.5 minutes)
        let wpm = Wpm::calculate(25, 1, 0, 0.5);
        assert_eq!(wpm.raw, 10.0); // (25/5) / 0.5 = 10
        assert_eq!(wpm.corrected, 8.0); // 10 - (1/0.5) = 8

        let ipm = Ipm::calculate(30, 40, 0.5);
        assert_eq!(ipm.actual, 60.0);
        assert_eq!(ipm.raw, 80.0);
    }

    #[test]
    fn test_consistency_calculations() {
        // Test with consistent typing (low standard deviation)
        let consistent_raw = [50.0, 51.0, 49.0, 50.5, 49.5];
        let consistent_corrected = [48.0, 49.0, 47.0, 48.5, 47.5];
        let consistent_actual = [46.0, 47.0, 45.0, 46.5, 45.5];

        // Create mock Wpm measurements for testing
        let consistent_measurements: Vec<Wpm> = (0..5)
            .map(|i| Wpm {
                raw: consistent_raw[i],
                corrected: consistent_corrected[i],
                actual: consistent_actual[i],
            })
            .collect();

        let consistency = Consistency::calculate(&consistent_measurements);

        // Should have low standard deviation (more consistent) and high percentage
        assert!(consistency.raw_deviation < 1.0);
        assert!(consistency.corrected_deviation < 1.0);
        assert!(consistency.actual_deviation < 1.0);
        assert!(consistency.raw_percent > 90.0);
        assert!(consistency.corrected_percent > 90.0);
        assert!(consistency.actual_percent > 90.0);

        // Test with inconsistent typing (high standard deviation)
        let inconsistent_raw = [30.0, 60.0, 40.0, 70.0, 20.0];
        let inconsistent_corrected = [25.0, 55.0, 35.0, 65.0, 15.0];
        let inconsistent_actual = [20.0, 50.0, 30.0, 60.0, 10.0];

        let inconsistent_measurements: Vec<Wpm> = (0..5)
            .map(|i| Wpm {
                raw: inconsistent_raw[i],
                corrected: inconsistent_corrected[i],
                actual: inconsistent_actual[i],
            })
            .collect();

        let consistency = Consistency::calculate(&inconsistent_measurements);

        // Should have high standard deviation (less consistent) and lower percentage
        assert!(consistency.raw_deviation > 15.0);
        assert!(consistency.corrected_deviation > 15.0);
        assert!(consistency.actual_deviation > 15.0);

        // With coefficient of variation, consistency percentages depend on mean WPM
        // For inconsistent data with means around 40-44 WPM and ~18.5 std dev:
        assert!(consistency.raw_percent < 70.0); // CV ≈ 0.42 → ~58% consistency
        assert!(consistency.corrected_percent < 60.0); // CV ≈ 0.47 → ~52% consistency  
        assert!(consistency.actual_percent < 50.0); // CV ≈ 0.55 → ~45% consistency

        // Test with single measurement (should be 0 deviation, 100% consistency)
        let single_measurement = [Wpm {
            raw: 50.0,
            corrected: 48.0,
            actual: 46.0,
        }];
        let consistency = Consistency::calculate(&single_measurement);
        assert_eq!(consistency.raw_deviation, 0.0);
        assert_eq!(consistency.corrected_deviation, 0.0);
        assert_eq!(consistency.actual_deviation, 0.0);
        assert_eq!(consistency.raw_percent, 100.0);
        assert_eq!(consistency.corrected_percent, 100.0);
        assert_eq!(consistency.actual_percent, 100.0);

        // Test with no measurements (should be 0 deviation, 100% consistency)
        let empty_measurements = [];
        let consistency = Consistency::calculate(&empty_measurements);
        assert_eq!(consistency.raw_deviation, 0.0);
        assert_eq!(consistency.corrected_deviation, 0.0);
        assert_eq!(consistency.actual_deviation, 0.0);
        assert_eq!(consistency.raw_percent, 100.0);
        assert_eq!(consistency.corrected_percent, 100.0);
        assert_eq!(consistency.actual_percent, 100.0);
    }

    #[test]
    fn test_consistency_edge_cases() {
        // Test with zero WPM values (should handle gracefully)
        let zero_wpm_measurements = [
            Wpm {
                raw: 0.0,
                corrected: 0.0,
                actual: 0.0,
            },
            Wpm {
                raw: 0.0,
                corrected: 0.0,
                actual: 0.0,
            },
        ];
        let consistency = Consistency::calculate(&zero_wpm_measurements);
        assert_eq!(consistency.raw_deviation, 0.0);
        assert_eq!(consistency.raw_percent, 100.0); // Zero mean should give 100% consistency
        assert_eq!(consistency.corrected_percent, 100.0);
        assert_eq!(consistency.actual_percent, 100.0);

        // Test with mixed zero/non-zero values
        let mixed_measurements = [
            Wpm {
                raw: 0.0,
                corrected: 0.0,
                actual: 0.0,
            },
            Wpm {
                raw: 50.0,
                corrected: 48.0,
                actual: 46.0,
            },
            Wpm {
                raw: 0.0,
                corrected: 0.0,
                actual: 0.0,
            },
        ];
        let consistency = Consistency::calculate(&mixed_measurements);
        assert!(consistency.raw_deviation > 20.0); // High deviation due to variance
        // Percentages depend on mean, should be lower due to high CV
        assert!(consistency.raw_percent < 50.0);
        assert!(consistency.corrected_percent < 50.0);
        assert!(consistency.actual_percent < 50.0);

        // Test identical measurements (zero standard deviation)
        let identical_measurements = [
            Wpm {
                raw: 60.0,
                corrected: 58.0,
                actual: 56.0,
            },
            Wpm {
                raw: 60.0,
                corrected: 58.0,
                actual: 56.0,
            },
            Wpm {
                raw: 60.0,
                corrected: 58.0,
                actual: 56.0,
            },
        ];
        let consistency = Consistency::calculate(&identical_measurements);
        assert_eq!(consistency.raw_deviation, 0.0);
        assert_eq!(consistency.corrected_deviation, 0.0);
        assert_eq!(consistency.actual_deviation, 0.0);
        assert_eq!(consistency.raw_percent, 100.0);
        assert_eq!(consistency.corrected_percent, 100.0);
        assert_eq!(consistency.actual_percent, 100.0);
    }

    #[test]
    fn test_consistency_boundary_conditions() {
        // Test very high CV (should give 0% consistency)
        let very_inconsistent = [
            Wpm {
                raw: 1.0,
                corrected: 1.0,
                actual: 1.0,
            }, // Very low
            Wpm {
                raw: 100.0,
                corrected: 98.0,
                actual: 96.0,
            }, // Very high
            Wpm {
                raw: 1.0,
                corrected: 1.0,
                actual: 1.0,
            }, // Very low again
        ];
        let consistency = Consistency::calculate(&very_inconsistent);
        assert!(consistency.raw_deviation > 45.0); // Very high std dev
        assert_eq!(consistency.raw_percent, 0.0); // CV > 1.0 should give 0%
        assert_eq!(consistency.corrected_percent, 0.0);
        assert_eq!(consistency.actual_percent, 0.0);

        // Test CV near 1.0 boundary
        let near_boundary = [
            Wpm {
                raw: 20.0,
                corrected: 18.0,
                actual: 16.0,
            },
            Wpm {
                raw: 40.0,
                corrected: 38.0,
                actual: 36.0,
            },
            Wpm {
                raw: 20.0,
                corrected: 18.0,
                actual: 16.0,
            },
            Wpm {
                raw: 40.0,
                corrected: 38.0,
                actual: 36.0,
            },
        ];
        let consistency = Consistency::calculate(&near_boundary);
        // Should have some consistency but not much (CV ≈ 0.33)
        assert!(consistency.raw_percent > 50.0 && consistency.raw_percent < 80.0);
        assert!(consistency.corrected_percent > 50.0 && consistency.corrected_percent < 80.0);
        assert!(consistency.actual_percent > 50.0 && consistency.actual_percent < 80.0);
    }

    #[test]
    fn test_consistency_realistic_patterns() {
        // Test gradual improvement over time
        let improving_pattern = [
            Wpm {
                raw: 30.0,
                corrected: 28.0,
                actual: 26.0,
            },
            Wpm {
                raw: 35.0,
                corrected: 33.0,
                actual: 31.0,
            },
            Wpm {
                raw: 40.0,
                corrected: 38.0,
                actual: 36.0,
            },
            Wpm {
                raw: 45.0,
                corrected: 43.0,
                actual: 41.0,
            },
            Wpm {
                raw: 50.0,
                corrected: 48.0,
                actual: 46.0,
            },
        ];
        let consistency = Consistency::calculate(&improving_pattern);
        // Should have moderate consistency (steady improvement)
        assert!(consistency.raw_deviation > 5.0 && consistency.raw_deviation < 10.0);
        assert!(consistency.raw_percent > 70.0 && consistency.raw_percent < 90.0);

        // Test sporadic performance (realistic inconsistency)
        let sporadic_pattern = [
            Wpm {
                raw: 45.0,
                corrected: 43.0,
                actual: 41.0,
            },
            Wpm {
                raw: 50.0,
                corrected: 48.0,
                actual: 46.0,
            },
            Wpm {
                raw: 35.0,
                corrected: 33.0,
                actual: 31.0,
            }, // Sudden drop
            Wpm {
                raw: 55.0,
                corrected: 53.0,
                actual: 51.0,
            }, // Recovery
            Wpm {
                raw: 48.0,
                corrected: 46.0,
                actual: 44.0,
            },
            Wpm {
                raw: 42.0,
                corrected: 40.0,
                actual: 38.0,
            },
        ];
        let consistency = Consistency::calculate(&sporadic_pattern);
        // Should show lower consistency due to sporadic performance
        assert!(consistency.raw_deviation > 5.0);
        assert!(consistency.raw_percent < 90.0); // Moderate inconsistency

        // Test beginner vs expert consistency patterns
        let beginner_pattern = [
            Wpm {
                raw: 15.0,
                corrected: 12.0,
                actual: 10.0,
            },
            Wpm {
                raw: 25.0,
                corrected: 20.0,
                actual: 15.0,
            },
            Wpm {
                raw: 12.0,
                corrected: 8.0,
                actual: 5.0,
            },
            Wpm {
                raw: 30.0,
                corrected: 25.0,
                actual: 20.0,
            },
        ];
        let beginner_consistency = Consistency::calculate(&beginner_pattern);

        let expert_pattern = [
            Wpm {
                raw: 85.0,
                corrected: 83.0,
                actual: 81.0,
            },
            Wpm {
                raw: 87.0,
                corrected: 85.0,
                actual: 83.0,
            },
            Wpm {
                raw: 83.0,
                corrected: 81.0,
                actual: 79.0,
            },
            Wpm {
                raw: 89.0,
                corrected: 87.0,
                actual: 85.0,
            },
        ];
        let expert_consistency = Consistency::calculate(&expert_pattern);

        // Expert should have better consistency (lower CV)
        assert!(expert_consistency.raw_percent > beginner_consistency.raw_percent);
        assert!(expert_consistency.corrected_percent > beginner_consistency.corrected_percent);
        assert!(expert_consistency.actual_percent > beginner_consistency.actual_percent);
    }
}
