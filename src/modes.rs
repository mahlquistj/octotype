use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::sources::SourceArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub name: String,
    pub description: Option<String>,
    pub source: Option<String>,
    pub parameters: HashMap<String, ParameterDefinition>,
    pub conditions: HashMap<String, ConditionDefinition>,
    pub source_overrides: HashMap<String, SourceOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceOverride {
    pub args: Vec<String>,
}

// Parameter definitions for customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterDefinition {
    Select {
        options: Vec<String>,
        default: String,
    },
    MultiSelect {
        options: Vec<String>,
        default: Vec<String>,
        min_selections: Option<usize>,
        max_selections: Option<usize>,
    },
    Toggle {
        default: bool,
        label: String,
    },
    Text {
        default: String,
        max_length: Option<usize>,
        pattern: Option<String>, // Regex validation
    },
}

// Condition definitions for win-conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionDefinition {
    Range {
        min: i32,
        max: Option<i32>,  // None = unbounded range
        default: i32,
        step: Option<i32>,
        suffix: Option<String>, // "seconds", "words", etc.
    },
    FloatRange {
        min: f64,
        max: f64,
        default: f64,
        step: f64,
        suffix: Option<String>, // "% accuracy", etc.
    },
}

// Runtime parameter values (customization)
#[derive(Debug, Clone)]
pub struct ParameterValues {
    values: HashMap<String, ParameterValue>,
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
    String(String),
    StringList(Vec<String>),
    Boolean(bool),
}

impl ParameterValues {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }
    
    pub fn set_string(&mut self, key: String, value: String) {
        self.values.insert(key, ParameterValue::String(value));
    }
    
    pub fn set_string_list(&mut self, key: String, value: Vec<String>) {
        self.values.insert(key, ParameterValue::StringList(value));
    }
    
    pub fn set_boolean(&mut self, key: String, value: bool) {
        self.values.insert(key, ParameterValue::Boolean(value));
    }
    
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.values.get(key)? {
            ParameterValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn get_string_list(&self, key: &str) -> Option<&[String]> {
        match self.values.get(key)? {
            ParameterValue::StringList(list) => Some(list),
            _ => None,
        }
    }
    
    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.values.get(key)? {
            ParameterValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

// Runtime condition values (win-conditions)
#[derive(Debug, Clone)]
pub struct ConditionValues {
    values: HashMap<String, ConditionValue>,
}

#[derive(Debug, Clone)]
pub enum ConditionValue {
    Integer(i32),
    Float(f64),
}

impl ConditionValues {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }
    
    pub fn set_integer(&mut self, key: String, value: i32) {
        self.values.insert(key, ConditionValue::Integer(value));
    }
    
    pub fn set_float(&mut self, key: String, value: f64) {
        self.values.insert(key, ConditionValue::Float(value));
    }
    
    pub fn get_integer(&self, key: &str) -> Option<i32> {
        match self.values.get(key)? {
            ConditionValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.values.get(key)? {
            ConditionValue::Float(f) => Some(*f),
            _ => None,
        }
    }
    
    pub fn get_duration(&self, key: &str) -> Option<Duration> {
        let seconds = self.get_integer(key)?;
        Some(Duration::from_secs(seconds as u64))
    }
}

// Fully resolved mode ready for use in typing session
#[derive(Debug, Clone)]
pub struct ResolvedModeConfig {
    pub name: String,
    pub source_name: Option<String>,
    pub parameter_values: ParameterValues,
    pub condition_values: ConditionValues,
    pub source_args: SourceArgs,
}

impl ResolvedModeConfig {
    pub fn from_mode_config(
        config: &ModeConfig, 
        parameter_values: ParameterValues,
        condition_values: ConditionValues
    ) -> Self {
        let mut source_args = SourceArgs::default();
        
        // Convert parameter values to source args
        if let Some(word_count) = condition_values.get_integer("word_count") {
            source_args.word_count = Some(word_count as usize);
        }
        
        if let Some(max_length) = parameter_values.get_string("max_word_length") {
            source_args.max_length = max_length.parse().ok();
        }
        
        source_args.difficulty = parameter_values.get_string("difficulty").map(|s| s.to_string());
        source_args.text_processing = parameter_values.get_string("text_processing").map(|s| s.to_string());
        
        Self {
            name: config.name.clone(),
            source_name: config.source.clone(),
            parameter_values,
            condition_values,
            source_args,
        }
    }
    
    pub fn is_complete(&self, session: &mut crate::page::session::TypingSession) -> bool {
        // Time-based completion
        if let Some(time_limit) = self.condition_values.get_duration("time_limit") {
            if let Some(start) = session.get_first_keypress() {
                if start.elapsed() >= time_limit {
                    return true;
                }
            }
        }
        
        // Word count completion
        if let Some(target_words) = self.condition_values.get_integer("word_count") {
            let typed_words = session.get_typed_word_count();
            if typed_words >= target_words as usize {
                return true;
            }
        }
        
        // Accuracy threshold completion
        if let Some(accuracy_threshold) = self.condition_values.get_float("accuracy_threshold") {
            let min_chars = self.condition_values.get_integer("min_chars").unwrap_or(50);
            if session.get_typed_char_count() >= min_chars as usize {
                let current_accuracy = session.calculate_accuracy();
                if current_accuracy >= accuracy_threshold {
                    return true;
                }
            }
        }
        
        // Default: all segments completed (for infinite modes)
        session.is_all_text_completed()
    }
}

pub struct ModeManager {
    modes: HashMap<String, ModeConfig>,
}

impl ModeManager {
    pub fn new() -> Self {
        Self {
            modes: HashMap::new(),
        }
    }
    
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.add_builtin_modes();
        manager
    }
    
    fn add_builtin_modes(&mut self) {
        // Built-in normal mode
        let mut normal_parameters = HashMap::new();
        normal_parameters.insert(
            "text_processing".to_string(),
            ParameterDefinition::Select {
                options: vec!["normal".to_string(), "no_punctuation".to_string(), "lowercase".to_string()],
                default: "normal".to_string(),
            }
        );
        
        let normal_mode = ModeConfig {
            name: "normal".to_string(),
            description: Some("Classic typing practice with quotes - no time limit".to_string()),
            source: Some("quotes".to_string()),
            parameters: normal_parameters,
            conditions: HashMap::new(),
            source_overrides: HashMap::new(),
        };
        self.modes.insert("normal".to_string(), normal_mode);
        
        // Built-in quick practice mode
        let mut quick_parameters = HashMap::new();
        quick_parameters.insert(
            "max_word_length".to_string(),
            ParameterDefinition::Text {
                default: "8".to_string(),
                max_length: Some(2),
                pattern: Some(r"^\d+$".to_string()),
            }
        );
        
        let mut quick_conditions = HashMap::new();
        quick_conditions.insert(
            "word_count".to_string(),
            ConditionDefinition::Range {
                min: 10,
                max: Some(100),
                default: 25,
                step: Some(5),
                suffix: Some("words".to_string()),
            }
        );
        
        let quick_practice_mode = ModeConfig {
            name: "quick_practice".to_string(),
            description: Some("Short practice session with word count goal".to_string()),
            source: Some("random_words".to_string()),
            parameters: quick_parameters,
            conditions: quick_conditions,
            source_overrides: HashMap::new(),
        };
        self.modes.insert("quick_practice".to_string(), quick_practice_mode);
        
        // Built-in timed challenge mode
        let mut timed_parameters = HashMap::new();
        timed_parameters.insert(
            "text_processing".to_string(),
            ParameterDefinition::Select {
                options: vec!["normal".to_string(), "no_punctuation".to_string()],
                default: "normal".to_string(),
            }
        );
        
        let mut timed_conditions = HashMap::new();
        timed_conditions.insert(
            "time_limit".to_string(),
            ConditionDefinition::Range {
                min: 30,
                max: Some(1800),
                default: 300,
                step: Some(30),
                suffix: Some("seconds".to_string()),
            }
        );
        
        let timed_challenge_mode = ModeConfig {
            name: "timed_challenge".to_string(),
            description: Some("Race against the clock".to_string()),
            source: Some("quotes".to_string()),
            parameters: timed_parameters,
            conditions: timed_conditions,
            source_overrides: HashMap::new(),
        };
        self.modes.insert("timed_challenge".to_string(), timed_challenge_mode);
    }
    
    pub fn load_from_config_dir(config_dir: &std::path::Path) -> Result<Self, ModeError> {
        let mut manager = Self::with_defaults();
        
        let modes_dir = config_dir.join("modes");
        if !modes_dir.exists() {
            return Ok(manager);
        }
        
        for entry in std::fs::read_dir(&modes_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |ext| ext == "toml") {
                let content = std::fs::read_to_string(&path)?;
                let config: ModeFileConfig = toml::from_str(&content)?;
                manager.modes.insert(config.mode.name.clone(), config.mode);
            }
        }
        
        Ok(manager)
    }
    
    pub fn get_mode(&self, name: &str) -> Option<&ModeConfig> {
        self.modes.get(name)
    }
    
    pub fn list_modes(&self) -> Vec<&str> {
        self.modes.keys().map(|s| s.as_str()).collect()
    }
    
    pub fn create_default_values(&self, mode_name: &str) -> Option<(ParameterValues, ConditionValues)> {
        let mode = self.get_mode(mode_name)?;
        
        let mut param_values = ParameterValues::new();
        let mut condition_values = ConditionValues::new();
        
        // Set parameter defaults
        for (key, param_def) in &mode.parameters {
            match param_def {
                ParameterDefinition::Select { default, .. } => {
                    param_values.set_string(key.clone(), default.clone());
                }
                ParameterDefinition::MultiSelect { default, .. } => {
                    param_values.set_string_list(key.clone(), default.clone());
                }
                ParameterDefinition::Toggle { default, .. } => {
                    param_values.set_boolean(key.clone(), *default);
                }
                ParameterDefinition::Text { default, .. } => {
                    param_values.set_string(key.clone(), default.clone());
                }
            }
        }
        
        // Set condition defaults
        for (key, condition_def) in &mode.conditions {
            match condition_def {
                ConditionDefinition::Range { default, .. } => {
                    condition_values.set_integer(key.clone(), *default);
                }
                ConditionDefinition::FloatRange { default, .. } => {
                    condition_values.set_float(key.clone(), *default);
                }
            }
        }
        
        Some((param_values, condition_values))
    }
}

#[derive(Deserialize)]
struct ModeFileConfig {
    mode: ModeConfig,
}

#[derive(Debug, Error)]
pub enum ModeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
}