use web_time::{Duration, Instant};

use crate::{Configuration, TempStatistics, Statistics};
use crate::text::CharacterResult;

/// Handles statistics tracking and timing
pub struct StatisticsTracker {
    stats: TempStatistics,
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
}

impl StatisticsTracker {
    pub fn new() -> Self {
        Self {
            stats: TempStatistics::default(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Get the current statistics
    pub fn statistics(&self) -> &TempStatistics {
        &self.stats
    }

    /// Update statistics based on input result
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

    /// Check if timing has started
    pub fn has_started(&self) -> bool {
        self.started_at.is_some()
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }

    /// Mark the typing session as completed
    pub fn mark_completed(&mut self) {
        if self.completed_at.is_none() {
            self.completed_at = Some(Instant::now());
        }
    }

    /// Check if the session has been completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get the total duration from start to completion
    pub fn total_duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    /// Finalize the statistics using the total duration and input length
    /// This consumes the StatisticsTracker and returns finalized Statistics
    pub fn finalize(self, input_len: usize) -> Result<Statistics, &'static str> {
        let total_duration = self.total_duration()
            .ok_or("Cannot finalize statistics: session not completed")?;
        
        Ok(self.stats.finalize(total_duration, input_len))
    }
}

impl Default for StatisticsTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Configuration;

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