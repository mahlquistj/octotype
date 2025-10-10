use gladius::statistics::Statistics;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use web_time::SystemTime;

use crate::page::session::Mode;

#[derive(Debug, Error)]
pub enum StatisticsError {
    #[error("Failed to create statistics directory: {0}")]
    CreateDirectory(std::io::Error),

    #[error("Failed to read statistics file: {0}")]
    ReadFile(std::io::Error),

    #[error("Failed to write statistics file: {0}")]
    WriteFile(std::io::Error),

    #[error("Failed to parse statistics: {0}")]
    Parse(serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub timestamp: SystemTime,
    pub session_id: String,
    pub session_config: SessionConfig,
    pub statistics: SerializableStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub mode_name: String,
    pub source_name: String,
    pub time_limit: Option<f64>,
    pub words_typed_limit: Option<usize>,
    pub allow_deletions: bool,
    pub allow_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableStatistics {
    pub duration: f64,
    pub wpm_actual: f64,
    pub wpm_raw: f64,
    pub accuracy_actual: f64,
    pub accuracy_raw: f64,
    pub consistency_actual_percent: f64,
    pub adds: usize,
    pub corrects: usize,
    pub errors: usize,
    pub corrections: usize,
    pub deletes: usize,
    pub wrong_deletes: usize,
}

impl From<&Statistics> for SerializableStatistics {
    fn from(stats: &Statistics) -> Self {
        Self {
            duration: stats.duration.as_secs_f64(),
            wpm_actual: stats.wpm.actual,
            wpm_raw: stats.wpm.raw,
            accuracy_actual: stats.accuracy.actual,
            accuracy_raw: stats.accuracy.raw,
            consistency_actual_percent: stats.consistency.actual_percent,
            adds: stats.counters.adds,
            corrects: stats.counters.corrects,
            errors: stats.counters.errors,
            corrections: stats.counters.corrections,
            deletes: stats.counters.deletes,
            wrong_deletes: stats.counters.wrong_deletes,
        }
    }
}

impl SessionConfig {
    pub fn from_mode(mode: &Mode, mode_name: String, source_name: String) -> Self {
        Self {
            mode_name,
            source_name,
            time_limit: mode.conditions.time.map(|d| d.as_secs_f64()),
            words_typed_limit: mode.conditions.words_typed,
            allow_deletions: mode.conditions.allow_deletions,
            allow_errors: mode.conditions.allow_errors,
        }
    }
}

#[derive(Debug)]
pub struct StatisticsManager {
    directory: PathBuf,
}

impl StatisticsManager {
    pub fn new(directory: PathBuf) -> Result<Self, StatisticsError> {
        if !directory.exists() {
            fs::create_dir_all(&directory).map_err(StatisticsError::CreateDirectory)?;
        }
        Ok(Self { directory })
    }

    pub fn save_session(
        &self,
        mode: &Mode,
        mode_name: String,
        source_name: String,
        statistics: &Statistics,
    ) -> Result<(), StatisticsError> {
        let session_stats = SessionStatistics {
            timestamp: SystemTime::now(),
            session_id: format!("{:?}", SystemTime::now()),
            session_config: SessionConfig::from_mode(mode, mode_name, source_name),
            statistics: SerializableStatistics::from(statistics),
        };

        let filename = format!(
            "session_{}.json",
            session_stats
                .timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        let file_path = self.directory.join(filename);

        let json = serde_json::to_string_pretty(&session_stats).map_err(StatisticsError::Parse)?;
        fs::write(file_path, json).map_err(StatisticsError::WriteFile)?;

        Ok(())
    }

    pub fn load_all_sessions(&self) -> Result<Vec<SessionStatistics>, StatisticsError> {
        let mut sessions = Vec::new();

        if !self.directory.exists() {
            return Ok(sessions);
        }

        let entries = fs::read_dir(&self.directory).map_err(StatisticsError::ReadFile)?;

        for entry in entries {
            let entry = entry.map_err(StatisticsError::ReadFile)?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let content = fs::read_to_string(&path).map_err(StatisticsError::ReadFile)?;
                match serde_json::from_str::<SessionStatistics>(&content) {
                    Ok(session) => sessions.push(session),
                    Err(_) => continue, // Skip invalid files
                }
            }
        }

        // Sort by timestamp (newest first)
        sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(sessions)
    }

    // Allow unused for future use case, as filters would be cool
    #[allow(unused)]
    pub fn load_sessions_for_config(
        &self,
        mode_name: &str,
        source_name: &str,
    ) -> Result<Vec<SessionStatistics>, StatisticsError> {
        let all_sessions = self.load_all_sessions()?;
        Ok(all_sessions
            .into_iter()
            .filter(|s| {
                s.session_config.mode_name == mode_name
                    && s.session_config.source_name == source_name
            })
            .collect())
    }
}
