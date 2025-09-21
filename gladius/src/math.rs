use crate::{AVERAGE_WORD_LENGTH, Float, Minutes};

/// Words Per Minute
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Wpm {
    /// The raw WPM describes how many words were typed, without accounting for errors or
    /// corrections. Just raw speed.
    pub raw: Float,
    /// The corrected WPM describes how many words were typed, taking only errors into account.
    pub corrected: Float,
    /// The actual WPM describes how many words were typed, taking errors and corrections into account.
    pub actual: Float,
}

impl Wpm {
    /// Calculate Words Per Minute
    ///
    /// * `characters` - How many characters where typed during `time`
    /// * `errors` - How many errors were made during `time`
    /// * `corrections` - How many corrections were made during `time`
    /// * `time` - How many minutes have gone by
    ///
    pub fn calculate(
        characters: usize,
        errors: usize,
        corrections: usize,
        minutes: Minutes,
    ) -> Self {
        let characters = characters as Float;
        let errors = errors as Float;
        let corrections = corrections as Float;

        #[cfg(not(feature = "f64"))]
        let minutes = minutes as Float;

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

/// Inputs Per Minute
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ipm {
    /// The raw ipm describes how many keypresses were made, including deletions and re-typing
    /// characters. Just raw input-speed.
    pub raw: Float,
    /// The actual ipm describes how many characters were added to the input, excluding deletions
    /// and re-typing characters.
    pub actual: Float,
}

impl Ipm {
    /// Calculates Inputs Per Minute
    ///
    /// * `actual_inputs` - How many characters where added to the input
    /// * `raw_inputs` - How many inputs happened during `time`, including deletions and re-typed
    ///   characters.
    /// * `time` - How many minutes have gone by
    ///
    pub fn calculate(actual_inputs: usize, raw_inputs: usize, minutes: Minutes) -> Self {
        let raw_inputs = raw_inputs as Float;
        let actual_inputs = actual_inputs as Float;

        #[cfg(not(feature = "f64"))]
        let minutes = minutes as Float;

        Self {
            raw: raw_inputs / minutes,
            actual: actual_inputs / minutes,
        }
    }
}

/// Typing accuracy
///
/// Accuracy describes the percentage of correctly typed characters.
///
/// The values in this struct are described as a percentage between 0.0 - 100.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Accuracy {
    /// Raw accuracy counts corrections as correct characters.
    pub raw: Float,
    /// Actual accuracy does not count corrections as correct characters
    pub actual: Float,
}

impl Accuracy {
    /// Calculate typing Accuracy
    ///
    /// * `input_len` - How many characters are typed currently
    /// * `total_errors` - The total amount of errors made
    /// * `total_corrections` - The total amount of corrections made
    ///
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

/// Typing consistency (standard deviation of WPM measurements)
///
/// Consistency describes the stability of typing speed over time.
/// Lower values indicate more consistent typing.
///
/// The values in this struct are described as a percentage between 0.0 - 100.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Consistency {
    /// Raw consistency standard deviation
    pub raw_deviation: Float,
    /// Raw consistency as percentage (0.0 - 100.0)
    pub raw_percent: Float,
    /// Corrected consistency standard deviation
    pub corrected_deviation: Float,
    /// Corrected consistency as percentage (0.0 - 100.0)
    pub corrected_percent: Float,
    /// Actual consistency standard deviation
    pub actual_deviation: Float,
    /// Actual consistency as percentage (0.0 - 100.0)
    pub actual_percent: Float,
}

impl Consistency {
    /// Calculate typing consistency
    ///
    /// * `raw_wpm_values` - Raw WPM measurements over time
    /// * `corrected_wpm_values` - Corrected WPM measurements over time
    /// * `actual_wpm_values` - Actual WPM measurements over time
    pub fn calculate(measurements: &[Wpm]) -> Self {
        let raw_wpm_values: Vec<Float> = measurements.iter().map(|m| m.raw).collect();
        let corrected_wpm_values: Vec<Float> = measurements.iter().map(|m| m.corrected).collect();
        let actual_wpm_values: Vec<Float> = measurements.iter().map(|m| m.actual).collect();
        let raw_deviation = Self::calculate_std_dev(&raw_wpm_values);
        let corrected_deviation = Self::calculate_std_dev(&corrected_wpm_values);
        let actual_deviation = Self::calculate_std_dev(&actual_wpm_values);

        Self {
            raw_deviation,
            raw_percent: Self::cv_to_percentage(raw_deviation, Self::calculate_mean(&raw_wpm_values)),
            corrected_deviation,
            corrected_percent: Self::cv_to_percentage(corrected_deviation, Self::calculate_mean(&corrected_wpm_values)),
            actual_deviation,
            actual_percent: Self::cv_to_percentage(actual_deviation, Self::calculate_mean(&actual_wpm_values)),
        }
    }

    fn calculate_std_dev(values: &[Float]) -> Float {
        if values.len() <= 1 {
            return 0.0;
        }

        let mean = values.iter().sum::<Float>() / values.len() as Float;
        let variance = values
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<Float>()
            / values.len() as Float;

        variance.sqrt()
    }

    fn calculate_mean(values: &[Float]) -> Float {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<Float>() / values.len() as Float
        }
    }

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
        let consistent_raw = vec![50.0, 51.0, 49.0, 50.5, 49.5];
        let consistent_corrected = vec![48.0, 49.0, 47.0, 48.5, 47.5];
        let consistent_actual = vec![46.0, 47.0, 45.0, 46.5, 45.5];

        // Create mock Wpm measurements for testing
        let consistent_measurements: Vec<Wpm> = (0..5).map(|i| Wpm {
            raw: consistent_raw[i],
            corrected: consistent_corrected[i],
            actual: consistent_actual[i],
        }).collect();
        
        let consistency = Consistency::calculate(&consistent_measurements);

        // Should have low standard deviation (more consistent) and high percentage
        assert!(consistency.raw_deviation < 1.0);
        assert!(consistency.corrected_deviation < 1.0);
        assert!(consistency.actual_deviation < 1.0);
        assert!(consistency.raw_percent > 90.0);
        assert!(consistency.corrected_percent > 90.0);
        assert!(consistency.actual_percent > 90.0);

        // Test with inconsistent typing (high standard deviation)
        let inconsistent_raw = vec![30.0, 60.0, 40.0, 70.0, 20.0];
        let inconsistent_corrected = vec![25.0, 55.0, 35.0, 65.0, 15.0];
        let inconsistent_actual = vec![20.0, 50.0, 30.0, 60.0, 10.0];

        let inconsistent_measurements: Vec<Wpm> = (0..5).map(|i| Wpm {
            raw: inconsistent_raw[i],
            corrected: inconsistent_corrected[i],
            actual: inconsistent_actual[i],
        }).collect();
        
        let consistency = Consistency::calculate(&inconsistent_measurements);

        // Should have high standard deviation (less consistent) and lower percentage
        assert!(consistency.raw_deviation > 15.0);
        assert!(consistency.corrected_deviation > 15.0);
        assert!(consistency.actual_deviation > 15.0);
        
        // With coefficient of variation, consistency percentages depend on mean WPM
        // For inconsistent data with means around 40-44 WPM and ~18.5 std dev:
        assert!(consistency.raw_percent < 70.0);     // CV ≈ 0.42 → ~58% consistency
        assert!(consistency.corrected_percent < 60.0); // CV ≈ 0.47 → ~52% consistency  
        assert!(consistency.actual_percent < 50.0);    // CV ≈ 0.55 → ~45% consistency

        // Test with single measurement (should be 0 deviation, 100% consistency)
        let single_measurement = vec![Wpm { raw: 50.0, corrected: 48.0, actual: 46.0 }];
        let consistency = Consistency::calculate(&single_measurement);
        assert_eq!(consistency.raw_deviation, 0.0);
        assert_eq!(consistency.corrected_deviation, 0.0);
        assert_eq!(consistency.actual_deviation, 0.0);
        assert_eq!(consistency.raw_percent, 100.0);
        assert_eq!(consistency.corrected_percent, 100.0);
        assert_eq!(consistency.actual_percent, 100.0);

        // Test with no measurements (should be 0 deviation, 100% consistency)
        let empty_measurements: Vec<Wpm> = vec![];
        let consistency = Consistency::calculate(&empty_measurements);
        assert_eq!(consistency.raw_deviation, 0.0);
        assert_eq!(consistency.corrected_deviation, 0.0);
        assert_eq!(consistency.actual_deviation, 0.0);
        assert_eq!(consistency.raw_percent, 100.0);
        assert_eq!(consistency.corrected_percent, 100.0);
        assert_eq!(consistency.actual_percent, 100.0);
    }
}
