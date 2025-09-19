use std::collections::HashMap;

pub use web_time::{Duration, Instant};

use crate::{CharacterResult, Float, State, Timestamp, Word};

mod math;

pub use math::*;

pub struct Input {
    pub timestamp: Timestamp,
    pub char: char,
    pub result: CharacterResult,
}

pub struct Speed {
    pub words_per_minute_actual: Float,
    pub words_per_minute_raw: Float,
    pub inputs_per_minute: Float,
}

pub struct Measurement {
    pub timestamp: Timestamp,
    pub speed: Speed,
    pub accuracy: Float,
}

pub struct Statistics {
    // Final stats
    pub wpm: Speed,
    pub accuracy: Float,
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
    pub total_adds: usize,
    pub total_deletes: usize,
    pub total_errors: usize,
    pub total_corrects: usize,
    pub total_corrections: usize,
    pub total_wrong_deletes: usize,

    // Meta
    last_measurement: Option<Timestamp>,
    inputs_since_last_measurement: usize,
    errors_since_last_measurement: usize,
}

impl TempStatistics {
    pub(crate) fn update(
        &mut self,
        char: char,
        result: CharacterResult,
        input_len: usize,
        timestamp: Timestamp,
    ) {
        // Update input history and counters
        self.update_from_result(char, result, timestamp);

        // Only poll for measurements each second
        if let Some(last_timestamp) = self.last_measurement {
            let time = timestamp - last_timestamp;
            if time >= 1.0 {
                self.measure(time, input_len.abs_diff(self.inputs_since_last_measurement));
            }
        } else if timestamp >= 1.0 {
            self.last_measurement = Some(timestamp);
            self.measure(
                timestamp,
                input_len.abs_diff(self.inputs_since_last_measurement),
            );
        }
    }

    fn measure(&mut self, time: Timestamp, characters_typed: usize) {
        // reset error counter after measure
        self.errors_since_last_measurement = 0;
    }

    fn update_from_result(&mut self, char: char, result: CharacterResult, timestamp: Timestamp) {
        match result {
            CharacterResult::Deleted(state) => {
                self.total_deletes += 1;
                if matches!(state, State::Correct | State::Corrected) {
                    self.total_wrong_deletes += 1
                }
            }
            CharacterResult::Wrong => {
                self.total_errors += 1;
                self.total_adds += 1;
                self.errors_since_last_measurement += 1;
            }
            CharacterResult::Corrected => {
                self.total_corrections += 1;
                self.total_adds += 1;
            }
            CharacterResult::Correct => {
                self.total_corrects += 1;
                self.total_adds += 1
            }
        }
        self.input_history.push(Input {
            timestamp,
            char,
            result,
        });
    }
}
