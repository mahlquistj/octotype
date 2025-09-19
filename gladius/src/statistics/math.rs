use crate::{AVERAGE_WORD_LENGTH, Float, Minutes};

/// Words Per Minute
pub struct Wpm {
    /// The raw WPM describes how many words were typed, without accounting for errors or
    /// corrections. Just raw speed.
    raw: Float,
    /// The corrected WPM describes how many words were typed, taking only errors into account.
    corrected: Float,
    /// The actual WPM describes how many words were typed, taking errors and corrections into account.
    actual: Float,
}

impl Wpm {
    /// Calculate Words Per Minute
    ///
    /// * `characters` - How many characters where typed during `time`
    /// * `errors` - How many errors were made during `time`
    /// * `corrections` - How many corrections were made during `time`
    /// * `time` - How many minutes have gone by
    ///
    pub fn new(characters: usize, errors: usize, corrections: usize, time: Minutes) -> Self {
        let characters = characters as Float;
        let errors = errors as Float;
        let corrections = corrections as Float;

        #[cfg(not(feature = "f64"))]
        let time = time as Float;

        // Errors Per Minute
        let epm = errors / time;
        // Corrections and Errors Per Minute
        let cepm = (corrections + errors) / time;

        // Raw WPM
        let raw = (characters / AVERAGE_WORD_LENGTH as Float) / time;

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
pub struct Ipm {
    /// The raw ipm describes how many keypresses were made, including deletions and re-typing
    /// characters. Just raw input-speed.
    raw: Float,
    /// The actual ipm describes how many characters were added to the input, excluding deletions
    /// and re-typing characters.
    actual: Float,
}

impl Ipm {
    /// Calculates Inputs Per Minute
    ///
    /// * `actual_inputs` - How many characters where added to the input
    /// * `raw_inputs` - How many inputs happened during `time`, including deletions and re-typed
    /// characters.
    /// * `time` - How many minutes have gone by
    ///
    pub fn new(actual_inputs: usize, raw_inputs: usize, time: Minutes) -> Self {
        let raw_inputs = raw_inputs as Float;
        let actual_inputs = actual_inputs as Float;

        #[cfg(not(feature = "f64"))]
        let time = time as Float;

        Self {
            raw: raw_inputs / time,
            actual: actual_inputs / time,
        }
    }
}

/// Typing accuracy
///
/// Accuracy describes the percentage of correctly typed characters.
///
/// The values in this struct are described as a percentage between 0.0 - 100.0.
pub struct Accuracy {
    /// Raw accuracy counts corrections as correct characters.
    raw: Float,
    /// Actual accuracy does not count corrections as correct characters
    actual: Float,
}

impl Accuracy {
    /// Calculate typing Accuracy
    ///
    /// * `input_len` - How many characters are typed currently
    /// * `total_errors` - The total amount of errors made
    /// * `total_corrections` - The total amount of corrections made
    ///
    /// Panics if `input_len` is `0`.
    pub fn new(input_len: usize, total_errors: usize, total_corrections: usize) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_calculations() {
        // Test basic WPM calculation: 50 chars, 0 errors, 0 corrections, 1 minute
        // Expected: 50 / 5 = 10 WPM (assuming AVERAGE_WORD_LENGTH is 5)
        let wpm = Wpm::new(50, 0, 0, 1.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 10.0);
        assert_eq!(wpm.actual, 10.0);

        // Test with errors: 50 chars, 2 errors, 0 corrections, 1 minute
        let wpm = Wpm::new(50, 2, 0, 1.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 8.0); // 10 - 2 errors
        assert_eq!(wpm.actual, 8.0); // 10 - (2 errors + 0 corrections)

        // Test with errors and corrections: 50 chars, 3 errors, 1 correction, 1 minute
        let wpm = Wpm::new(50, 3, 1, 1.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 7.0); // 10 - 3 errors
        assert_eq!(wpm.actual, 6.0); // 10 - (3 errors + 1 correction)

        // Test with different time: 100 chars, 0 errors, 0 corrections, 2 minutes
        let wpm = Wpm::new(100, 0, 0, 2.0);
        assert_eq!(wpm.raw, 10.0);
        assert_eq!(wpm.corrected, 10.0);
        assert_eq!(wpm.actual, 10.0);
    }

    #[test]
    fn test_ipm_calculations() {
        // Test basic IPM: 60 actual inputs, 80 raw inputs, 1 minute
        let ipm = Ipm::new(60, 80, 1.0);
        assert_eq!(ipm.actual, 60.0);
        assert_eq!(ipm.raw, 80.0);

        // Test with different time: 120 actual inputs, 150 raw inputs, 2 minutes
        let ipm = Ipm::new(120, 150, 2.0);
        assert_eq!(ipm.actual, 60.0);
        assert_eq!(ipm.raw, 75.0);

        // Test where actual equals raw (no deletions/retyping)
        let ipm = Ipm::new(50, 50, 1.0);
        assert_eq!(ipm.actual, 50.0);
        assert_eq!(ipm.raw, 50.0);
    }

    #[test]
    fn test_accuracy_calculations() {
        // Test perfect accuracy: 100 chars, 0 errors, 0 corrections
        let accuracy = Accuracy::new(100, 0, 0);
        assert_eq!(accuracy.raw, 100.0);
        assert_eq!(accuracy.actual, 100.0);

        // Test with errors but no corrections: 100 chars, 5 errors, 0 corrections
        let accuracy = Accuracy::new(100, 5, 0);
        assert_eq!(accuracy.raw, 95.0);
        assert_eq!(accuracy.actual, 95.0);

        // Test with errors and corrections: 100 chars, 10 errors, 6 corrections
        let accuracy = Accuracy::new(100, 10, 6);
        assert_eq!(accuracy.raw, 90.0); // (1 - 10/100) * 100
        assert_eq!(accuracy.actual, 96.0); // (1 - (10-6)/100) * 100

        // Test edge case: more corrections than errors
        let accuracy = Accuracy::new(100, 5, 8);
        assert_eq!(accuracy.raw, 95.0);
        assert_eq!(accuracy.actual, 100.0); // Clamped to 0 errors
    }

    #[test]
    fn test_fractional_time() {
        // Test with 30 seconds (0.5 minutes)
        let wpm = Wpm::new(25, 1, 0, 0.5);
        assert_eq!(wpm.raw, 10.0); // (25/5) / 0.5 = 10
        assert_eq!(wpm.corrected, 8.0); // 10 - (1/0.5) = 8

        let ipm = Ipm::new(30, 40, 0.5);
        assert_eq!(ipm.actual, 60.0);
        assert_eq!(ipm.raw, 80.0);
    }
}
