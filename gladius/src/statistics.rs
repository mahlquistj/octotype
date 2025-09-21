use std::collections::HashMap;

pub use web_time::{Duration, Instant};

use crate::{
    Accuracy, CharacterResult, Configuration, Consistency, Ipm, State, Timestamp, Word, Wpm,
};

pub struct Input {
    pub timestamp: Timestamp,
    pub char: char,
    pub result: CharacterResult,
}

pub struct Measurement {
    pub timestamp: Timestamp,
    pub wpm: Wpm,
    pub ipm: Ipm,
    pub accuracy: Accuracy,
    pub consistency: Consistency,
}

impl Measurement {
    /// Create a new measurement from current statistics
    pub fn new(
        timestamp: Timestamp,
        input_len: usize,
        previous_measurements: &[Measurement],
        input_history: &[Input],
        adds: usize,
        errors: usize,
        corrections: usize,
    ) -> Self {
        let minutes = timestamp / 60.0;

        let wpm = Wpm::calculate(input_history.len(), errors, corrections, minutes);
        let ipm = Ipm::calculate(adds, input_history.len(), minutes);
        let accuracy = Accuracy::calculate(input_len, errors, corrections);

        // Calculate consistency - create a temporary Vec with all WPM measurements
        let all_wpm_measurements: Vec<Wpm> = previous_measurements
            .iter()
            .map(|m| m.wpm)
            .chain(std::iter::once(wpm))
            .collect();

        let consistency = Consistency::calculate(&all_wpm_measurements);

        Self {
            timestamp,
            wpm,
            ipm,
            accuracy,
            consistency,
        }
    }
}

#[derive(Default)]
pub struct CounterData {
    pub char_errors: HashMap<char, usize>,
    pub word_errors: HashMap<Word, usize>,
    pub adds: usize,
    pub deletes: usize,
    pub errors: usize,
    pub corrects: usize,
    pub corrections: usize,
    pub wrong_deletes: usize,
}

pub struct Statistics {
    // Final stats
    pub wpm: Wpm,
    pub ipm: Ipm,
    pub accuracy: Accuracy,
    pub consistency: Consistency,

    // Historical data
    pub measurements: Vec<Measurement>,
    pub input_history: Vec<Input>,

    // Counters
    pub counters: CounterData,
}

#[derive(Default)]
pub struct TempStatistics {
    // Historical data
    pub measurements: Vec<Measurement>,
    pub input_history: Vec<Input>,

    // Counters
    pub counters: CounterData,

    // Meta
    last_measurement: Option<Timestamp>,
}

impl TempStatistics {
    /// Update the statistics
    pub(crate) fn update(
        &mut self,
        char: char,
        result: CharacterResult,
        input_len: usize,
        elapsed: Duration,
        config: &Configuration,
    ) {
        let timestamp = elapsed.as_secs_f64();
        // Update input history and counters
        self.update_from_result(char, result, timestamp);

        // Only poll for measurements each second
        if let Some(last_timestamp) = self.last_measurement {
            let time = timestamp - last_timestamp;
            if time >= config.measurement_interval_seconds {
                self.measure(time, input_len);
            }
        } else if timestamp >= config.measurement_interval_seconds {
            self.last_measurement = Some(timestamp);
            self.measure(timestamp, input_len);
        }
    }

    /// Grab a measurement for the statistics
    fn measure(&mut self, timestamp: Timestamp, input_len: usize) {
        let measurement = Measurement::new(
            timestamp,
            input_len,
            &self.measurements,
            &self.input_history,
            self.counters.adds,
            self.counters.errors,
            self.counters.corrections,
        );
        self.measurements.push(measurement);
    }

    /// Update counters and input history
    fn update_from_result(&mut self, char: char, result: CharacterResult, timestamp: Timestamp) {
        match result {
            CharacterResult::Deleted(state) => {
                self.counters.deletes += 1;
                if matches!(state, State::Correct | State::Corrected) {
                    self.counters.wrong_deletes += 1
                }
            }
            CharacterResult::Wrong => {
                self.counters.errors += 1;
                self.counters.adds += 1;
                *self.counters.char_errors.entry(char).or_insert(0) += 1;
            }
            CharacterResult::Corrected => {
                self.counters.corrections += 1;
                self.counters.adds += 1;
            }
            CharacterResult::Correct => {
                self.counters.corrects += 1;
                self.counters.adds += 1;
            }
        }
        self.input_history.push(Input {
            timestamp,
            char,
            result,
        });
    }

    /// Finalize the temporary statistics and return the final Statistics
    pub fn finalize(self, total_time: Timestamp, input_len: usize) -> Statistics {
        let Self {
            measurements,
            input_history,
            counters,
            ..
        } = self;

        let Measurement {
            wpm,
            ipm,
            accuracy,
            consistency,
            ..
        } = Measurement::new(
            total_time,
            input_len,
            &measurements,
            &input_history,
            counters.adds,
            counters.errors,
            counters.corrections,
        );

        Statistics {
            wpm,
            ipm,
            accuracy,
            consistency,
            measurements,
            input_history,
            counters,
        }
    }
}
