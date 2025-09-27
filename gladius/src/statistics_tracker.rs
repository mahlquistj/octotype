//! # Statistics Tracker Module - Session Timing and Statistics Coordination
//!
//! This module provides the high-level interface for tracking typing performance
//! during active sessions. It coordinates timing, statistics collection, and
//! session lifecycle management.
//!
//! ## Key Features
//!
//! - **Automatic Timing**: Starts timing on first keystroke, tracks session duration
//! - **Statistics Integration**: Coordinates with TempStatistics for data collection
//! - **Session Lifecycle**: Manages start, update, completion, and finalization phases
//! - **Real-time Updates**: Provides current statistics during active typing
//!
//! ## Session Lifecycle
//!
#![doc = simple_mermaid::mermaid!("../diagrams/session_lifecycle.mmd")]
//!
//! ## Usage Pattern
//!
//! ```rust
//! use gladius::statistics_tracker::StatisticsTracker;
//! use gladius::config::Configuration;
//! use gladius::CharacterResult;
//!
//! let mut tracker = StatisticsTracker::new();
//! let config = Configuration::default();
//!
//! // Process typing events
//! tracker.update('h', CharacterResult::Correct, 1, &config);
//! tracker.update('e', CharacterResult::Correct, 2, &config);
//!
//! // Mark session complete and get final statistics.
//! tracker.mark_completed();
//! // The tracker does not handle the input, so it needs to know the final input length
//! let final_stats = tracker.finalize(2).unwrap(); // 2 = final input length
//! ```

use web_time::{Duration, Instant};

use crate::CharacterResult;
use crate::config::Configuration;
use crate::statistics::{Statistics, TempStatistics};

/// High-level statistics tracking coordinator for typing sessions
///
/// Manages the complete lifecycle of typing performance tracking, from session
/// initialization through finalization. Provides automatic timing and coordinates
/// with the underlying statistics collection system.
///
/// # Lifecycle States
///
/// - **Unstarted**: Created but no input received yet
/// - **Active**: Timing started, collecting statistics in real-time
/// - **Completed**: Session marked as finished, ready for finalization
/// - **Finalized**: Consumed to produce final Statistics (terminal state)
///
/// # Thread Safety
///
/// This structure is not thread-safe. Each typing session should have its own
/// StatisticsTracker instance on a single thread.
#[derive(Debug, Clone)]
pub struct StatisticsTracker {
    /// Underlying statistics accumulator
    stats: TempStatistics,
    /// When the typing session started (set on first keystroke)
    started_at: Option<Instant>,
    /// When the typing session was marked as complete
    completed_at: Option<Instant>,
}

impl StatisticsTracker {
    /// Create a new statistics tracker for a typing session
    ///
    /// Initializes the tracker in the unstarted state. Timing will begin
    /// automatically when the first keystroke is processed.
    pub fn new() -> Self {
        Self {
            stats: TempStatistics::default(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Get read-only access to the current statistics
    ///
    /// Provides access to real-time statistics during the typing session.
    /// Useful for displaying live WPM, accuracy, and other metrics.
    pub fn statistics(&self) -> &TempStatistics {
        &self.stats
    }

    /// Process a keystroke and update statistics
    ///
    /// Handles timing initialization, statistics updates, and measurements.
    /// Automatically starts timing on the first keystroke.
    ///
    /// # Parameters
    ///
    /// * `char` - The character that was typed
    /// * `result` - Whether it was correct, wrong, corrected, or deleted
    /// * `input_len` - Current length of the typed input
    /// * `config` - Configuration for measurement intervals and behavior
    ///
    /// # Timing Behavior
    ///
    /// - First call: Starts the session timer automatically
    /// - Subsequent calls: Updates elapsed time and processes statistics
    pub fn update(
        &mut self,
        char: char,
        result: CharacterResult,
        input_len: usize,
        config: &Configuration,
    ) {
        // Initialize timing on first input
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }

        // Safety: We just set started_at above if it was None
        let started_at = self.started_at.as_ref().unwrap();
        let elapsed = started_at.elapsed();

        self.stats.update(char, result, input_len, elapsed, config);
    }

    /// Check if the typing session has started
    ///
    /// Returns `true` if at least one keystroke has been processed.
    pub fn has_started(&self) -> bool {
        self.started_at.is_some()
    }

    /// Get the current elapsed time since the session started
    ///
    /// Returns `None` if the session hasn't started yet.
    pub fn elapsed(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }

    /// Mark the typing session as completed
    ///
    /// Records the completion time for final duration calculation.
    /// Can be called multiple times safely (subsequent calls are ignored).
    pub fn mark_completed(&mut self) {
        if self.completed_at.is_none() {
            self.completed_at = Some(Instant::now());
        }
    }

    /// Check if the session has been marked as completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get the total session duration
    ///
    /// Returns the duration from start to completion if both are recorded,
    /// or from start to now if session is active but not completed.
    pub fn total_duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            (Some(start), None) => Some(start.elapsed()),
            _ => None,
        }
    }

    /// Convert the tracker into final session statistics
    ///
    /// Consumes the tracker and produces comprehensive final statistics
    /// including all measurements, counters, and calculated metrics.
    ///
    /// # Parameters
    ///
    /// * `input_len` - The final length of the typed input
    ///
    /// # Returns
    ///
    /// `Ok(Statistics)` if successful, `Err` if the session was never started
    ///
    /// # Errors
    ///
    /// Returns an error if called before any keystrokes have been processed.
    /// The session must be started (but not necessarily completed) to finalize.
    pub fn finalize(self, input_len: usize) -> Statistics {
        let total_duration = self.total_duration().unwrap_or(Duration::ZERO);
        self.stats.finalize(total_duration, input_len)
    }
}

impl Default for StatisticsTracker {
    /// Create a new statistics tracker (same as `new()`)
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_tracker() {
        let mut stats_tracker = StatisticsTracker::new();
        let config = Configuration::default();

        // Initially no statistics
        let stats = stats_tracker.statistics();
        assert_eq!(stats.counters.adds, 0);
        assert_eq!(stats.counters.errors, 0);
        assert!(!stats_tracker.has_started());

        // Update with wrong character
        stats_tracker.update('x', CharacterResult::Wrong, 1, &config);
        let stats = stats_tracker.statistics();
        assert_eq!(stats.counters.adds, 1);
        assert_eq!(stats.counters.errors, 1);
        assert!(stats_tracker.has_started());

        // Update with correct character
        stats_tracker.update('b', CharacterResult::Correct, 2, &config);
        let stats = stats_tracker.statistics();
        assert_eq!(stats.counters.adds, 2);
        assert_eq!(stats.counters.errors, 1);

        // Check elapsed time is available
        assert!(stats_tracker.elapsed().is_some());
    }
}
