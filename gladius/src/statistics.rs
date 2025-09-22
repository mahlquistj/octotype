//! # Statistics Module - Typing Performance Data Collection and Analysis
//!
//! This module provides comprehensive data structures and algorithms for collecting,
//! processing, and analyzing typing performance statistics in real-time during
//! typing sessions.
//!
//! ## Architecture Overview
//!
//! The statistics system follows a multi-layered architecture:
//!
#![doc = simple_mermaid::mermaid!("../diagrams/statistics_architecture.mmd")]
//!
//! ## Key Components
//!
//! - **Input**: Individual keystroke events with timing and correctness
//! - **Measurement**: Point-in-time snapshots of all metrics
//! - **TempStatistics**: Accumulates data during active typing
//! - **Statistics**: Final session summary with complete analysis
//! - **CounterData**: Tracks various typing event counters
//!
//! ## Data Flow
//!
//! 1. **Event Collection**: Each keystroke generates an `Input` event
//! 2. **Real-time Processing**: `TempStatistics` updates counters and metrics
//! 3. **Periodic Sampling**: `Measurement` snapshots taken at intervals
//! 4. **Session Finalization**: Complete `Statistics` generated at end
//!
//! ## Performance Considerations
//!
//! - Measurements are taken at configurable intervals to balance accuracy vs. performance
//! - Consistency calculations use efficient Welford's algorithm for numerical stability
//! - Error tracking uses HashMap for efficient character-specific analysis

use std::collections::HashMap;

pub use web_time::{Duration, Instant};

use crate::{
    CharacterResult, State, Timestamp, Word,
    config::Configuration,
    math::{Accuracy, Consistency, Ipm, Wpm},
};

/// Individual keystroke event with timing and correctness information
///
/// Used to build the complete history of typing activity for analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input {
    /// Timestamp in seconds from session start
    pub timestamp: Timestamp,
    /// Character that was typed
    pub char: char,
    /// Whether the keystroke was correct, wrong, corrected, or deleted
    pub result: CharacterResult,
}

/// Point-in-time snapshot of all typing performance metrics
///
/// Measurements are taken at regular intervals during typing to track
/// performance changes over time and calculate consistency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// When this measurement was taken (seconds from session start)
    pub timestamp: Timestamp,
    /// Words per minute at this point in time
    pub wpm: Wpm,
    /// Inputs per minute at this point in time
    pub ipm: Ipm,
    /// Typing accuracy at this point in time
    pub accuracy: Accuracy,
    /// Typing consistency up to this point in time
    pub consistency: Consistency,
}

impl Measurement {
    /// Create a new measurement snapshot from current session data
    ///
    /// Calculates all performance metrics based on the current state of the typing session.
    /// Consistency is calculated using all previous measurements plus the current one.
    ///
    /// # Parameters
    ///
    /// * `timestamp` - Current time in seconds from session start
    /// * `input_len` - Current length of the typed input
    /// * `previous_measurements` - All measurements taken so far in this session
    /// * `input_history` - Complete history of keystrokes
    /// * `adds` - Total number of characters added (not including deletions)
    /// * `errors` - Total number of errors made
    /// * `corrections` - Total number of corrections made
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

/// Comprehensive counters for all typing events and errors
///
/// Tracks various statistics needed for performance analysis and detailed feedback.
/// Used internally by TempStatistics to accumulate data during typing sessions.
#[derive(Default, Debug, Clone)]
pub struct CounterData {
    /// Number of errors for each character (for targeted practice)
    pub char_errors: HashMap<char, usize>,
    /// Number of errors for each word (for word-level analysis)
    pub word_errors: HashMap<Word, usize>,
    /// Total characters added to the input (excluding deletions)
    pub adds: usize,
    /// Total delete operations performed
    pub deletes: usize,
    /// Total number of incorrect characters typed
    pub errors: usize,
    /// Total number of correct characters typed
    pub corrects: usize,
    /// Total number of corrections made (fixing previous errors)
    pub corrections: usize,
    /// Number of times correct characters were deleted (typing inefficiency)
    pub wrong_deletes: usize,
}

/// Complete statistical analysis of a finished typing session
///
/// Contains final performance metrics, historical data, and detailed counters.
/// Generated by finalizing a TempStatistics after the typing session ends.
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Final words per minute calculations (raw, corrected, actual)
    pub wpm: Wpm,
    /// Final inputs per minute calculations (raw, actual)
    pub ipm: Ipm,
    /// Final accuracy percentages (raw, actual)
    pub accuracy: Accuracy,
    /// Final consistency percentages and standard deviations
    pub consistency: Consistency,
    /// Total duration of the typing session
    pub duration: Duration,

    /// All measurements taken during the session (for trend analysis)
    pub measurements: Vec<Measurement>,
    /// Complete keystroke history (for detailed analysis)
    pub input_history: Vec<Input>,
    /// Detailed counters for all typing events
    pub counters: CounterData,
}

/// Real-time statistics accumulator for active typing sessions
///
/// Collects and processes typing events as they occur, taking periodic measurements
/// for consistency analysis. Designed for efficient real-time updates during typing.
#[derive(Default, Debug, Clone)]
pub struct TempStatistics {
    /// Measurements taken at regular intervals during the session
    pub measurements: Vec<Measurement>,
    /// Complete history of every keystroke in the session
    pub input_history: Vec<Input>,
    /// Running counters for all typing events and errors
    pub counters: CounterData,
    /// Timestamp of the last measurement (for interval tracking)
    last_measurement: Option<Timestamp>,
}

impl TempStatistics {
    /// Process a new keystroke event and update all statistics
    ///
    /// Updates counters, adds to input history, and takes a measurement
    /// if enough time has elapsed since the last one.
    ///
    /// # Parameters
    ///
    /// * `char` - The character that was typed
    /// * `result` - Whether it was correct, wrong, corrected, or deleted
    /// * `input_len` - Current length of the input text
    /// * `elapsed` - Time elapsed since session start
    /// * `config` - Configuration including measurement interval
    pub fn update(
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

        // Take measurement if enough time has elapsed
        if self.should_take_measurement(timestamp, config.measurement_interval_seconds) {
            self.take_measurement(timestamp, input_len);
        }
    }

    /// Check if enough time has elapsed to take a new measurement
    fn should_take_measurement(&self, current_timestamp: Timestamp, interval_seconds: f64) -> bool {
        match self.last_measurement {
            Some(last_timestamp) => current_timestamp - last_timestamp >= interval_seconds,
            None => current_timestamp >= interval_seconds,
        }
    }

    /// Take a measurement and update the last measurement timestamp
    fn take_measurement(&mut self, timestamp: Timestamp, input_len: usize) {
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
        self.last_measurement = Some(timestamp);
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

    /// Convert temporary statistics into final session statistics
    ///
    /// Calculates final metrics based on the complete session data and returns
    /// a comprehensive Statistics struct suitable for analysis and storage.
    pub fn finalize(self, duration: Duration, input_len: usize) -> Statistics {
        let total_time = duration.as_secs_f64();

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
            duration,
            measurements,
            input_history,
            counters,
        }
    }
}
