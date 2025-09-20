use std::collections::HashMap;

pub use web_time::{Duration, Instant};

use crate::{Accuracy, CharacterResult, Configuration, Float, Ipm, State, Timestamp, Word, Wpm};

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
}

pub struct Statistics {
    // Final stats
    pub wpm: Wpm,
    pub ipm: Ipm,
    pub accuracy: Accuracy,
    pub consistency: Float,

    // Historical data
    pub measurements: Vec<(Timestamp, Measurement)>,
    pub input_history: Vec<Input>,

    // Counters
    pub char_errors: HashMap<char, usize>,
    pub word_errors: HashMap<String, usize>,
    pub total_chars: usize,
    pub total_inputs: usize,
    pub total_adds: usize,
}

#[derive(Default)]
pub struct TempStatistics {
    // Historical data
    pub measurements: Vec<Measurement>,
    pub input_history: Vec<Input>,

    // Counters
    pub char_errors: HashMap<char, usize>,
    pub word_errors: HashMap<Word, usize>,
    // All of the counters below could technically be calculated directly from `input_history`, but
    // i think that it may be unnescessary overhead, during typing - I would rather use more memory
    pub adds: usize,
    pub deletes: usize,
    pub errors: usize,
    pub corrects: usize,
    pub corrections: usize,
    pub wrong_deletes: usize,

    // Meta
    last_measurement: Option<Timestamp>,
}

impl TempStatistics {
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
        let minutes = timestamp / 60.0;
        let wpm = Wpm::new(
            self.input_history.len(),
            self.errors,
            self.corrections,
            minutes,
        );
        let ipm = Ipm::new(self.adds, self.input_history.len(), minutes);
        let accuracy = Accuracy::new(input_len, self.errors, self.corrections);

        self.measurements.push(Measurement {
            timestamp,
            wpm,
            ipm,
            accuracy,
        });
    }

    /// Update counters and input history
    fn update_from_result(&mut self, char: char, result: CharacterResult, timestamp: Timestamp) {
        match result {
            CharacterResult::Deleted(state) => {
                self.deletes += 1;
                if matches!(state, State::Correct | State::Corrected) {
                    self.wrong_deletes += 1
                }
            }
            CharacterResult::Wrong => {
                self.errors += 1;
                self.adds += 1;
                self.char_errors
                    .entry(char)
                    .and_modify(|count| *count += 1)
                    .or_insert_with(|| 1);
            }
            CharacterResult::Corrected => {
                self.corrections += 1;
                self.adds += 1;
            }
            CharacterResult::Correct => {
                self.corrects += 1;
                self.adds += 1
            }
        }
        self.input_history.push(Input {
            timestamp,
            char,
            result,
        });
    }
}
