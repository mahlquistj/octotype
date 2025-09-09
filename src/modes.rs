use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

use crate::sources::{ParameterValues, Source};

#[derive(Debug, Error)]
pub enum ModeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeFileConfig {
    pub meta: ModeMeta,
    #[serde(default)]
    pub parameters: HashMap<String, crate::sources::ParameterDefinition>,
    #[serde(default)]
    pub conditions: HashMap<String, ConditionDefinition>,
    #[serde(default)]
    pub source_overrides: HashMap<String, HashMap<String, String>>,
    pub sources: Option<Vec<String>>, // Allowed sources for this mode
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeMeta {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ModeConfig {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, crate::sources::ParameterDefinition>,
    pub conditions: HashMap<String, ConditionDefinition>,
    pub source_overrides: HashMap<String, HashMap<String, String>>,
    pub allowed_sources: Option<Vec<String>>,
}

impl From<ModeFileConfig> for ModeConfig {
    fn from(file_config: ModeFileConfig) -> Self {
        Self {
            name: file_config.meta.name,
            description: file_config.meta.description,
            parameters: file_config.parameters,
            conditions: file_config.conditions,
            source_overrides: file_config.source_overrides,
            allowed_sources: file_config.sources,
        }
    }
}

// Condition definitions for session termination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionDefinition {
    Range {
        min: i32,
        max: Option<i32>, // None = unbounded range
        step: Option<i32>,
        #[serde(default)]
        default: Option<i32>, // Defaults to min if not set
    },
    Fixed(i32), // For conditions like "words_typed = 500"
}

// Runtime condition values (session termination conditions)
#[derive(Debug, Clone, Default)]
pub struct ConditionValues {
    values: HashMap<String, i32>,
}

impl ConditionValues {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set_integer(&mut self, key: String, value: i32) {
        self.values.insert(key, value);
    }

    pub fn get_integer(&self, key: &str) -> Option<i32> {
        self.values.get(key).copied()
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
    pub parameter_values: ParameterValues,
    pub condition_values: ConditionValues,
    pub source_name: String,
    pub source_parameters: ParameterValues,
}

impl ResolvedModeConfig {
    pub const fn new(
        name: String,
        parameter_values: ParameterValues,
        condition_values: ConditionValues,
        source_name: String,
        source_parameters: ParameterValues,
    ) -> Self {
        Self {
            name,
            parameter_values,
            condition_values,
            source_name,
            source_parameters,
        }
    }

    pub fn should_fetch_more_content(&self, session: &crate::page::session::TypingSession) -> bool {
        // If session is near completion and we have infinite sources, fetch more
        let remaining_chars = session.get_remaining_char_count();
        remaining_chars < 100 // Fetch more when less than 100 characters remain
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

    pub fn load_from_config_dir(config_dir: &std::path::Path) -> Result<Self, ModeError> {
        let mut manager = Self::with_defaults();

        let modes_dir = config_dir.join("modes");
        if !modes_dir.exists() {
            return Ok(manager);
        }

        for entry in std::fs::read_dir(&modes_dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                let content = std::fs::read_to_string(&path)?;
                let file_config: ModeFileConfig = toml::from_str(&content)?;
                let mode_config: ModeConfig = file_config.into();
                manager.modes.insert(mode_config.name.clone(), mode_config);
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

    pub fn create_default_values(
        &self,
        mode_name: &str,
    ) -> Option<(ParameterValues, ConditionValues)> {
        let mode = self.get_mode(mode_name)?;

        let mut param_values = ParameterValues::new();
        let mut condition_values = ConditionValues::new();

        // Set parameter defaults
        for (key, param_def) in &mode.parameters {
            match param_def {
                crate::sources::ParameterDefinition::Selection { default, .. } => {
                    param_values.set_string(key.clone(), default.clone());
                }
                crate::sources::ParameterDefinition::Range { min, default, .. } => {
                    let value = default.unwrap_or(*min);
                    param_values.set_integer(key.clone(), value);
                }
                crate::sources::ParameterDefinition::Toggle(default_value) => {
                    param_values.set_boolean(key.clone(), *default_value);
                }
            }
        }

        // Set condition defaults
        for (key, condition_def) in &mode.conditions {
            match condition_def {
                ConditionDefinition::Range { min, default, .. } => {
                    let value = default.unwrap_or(*min);
                    condition_values.set_integer(key.clone(), value);
                }
                ConditionDefinition::Fixed(value) => {
                    condition_values.set_integer(key.clone(), *value);
                }
            }
        }

        Some((param_values, condition_values))
    }

    fn add_builtin_modes(&mut self) {
        // Built-in normal mode
        let mut normal_parameters = HashMap::new();
        normal_parameters.insert(
            "text_processing".to_string(),
            crate::sources::ParameterDefinition::Selection {
                options: vec![
                    "normal".to_string(),
                    "no_punctuation".to_string(),
                    "lowercase".to_string(),
                ],
                default: "normal".to_string(),
            },
        );

        let normal_mode = ModeConfig {
            name: "normal".to_string(),
            description: "Classic typing practice with quotes - no time limit".to_string(),
            parameters: normal_parameters,
            conditions: HashMap::new(),
            source_overrides: HashMap::new(),
            allowed_sources: None,
        };
        self.modes.insert("normal".to_string(), normal_mode);

        // Built-in quick practice mode
        let mut quick_parameters = HashMap::new();
        quick_parameters.insert(
            "word_count".to_string(),
            crate::sources::ParameterDefinition::Range {
                min: 10,
                max: 100,
                step: 5,
                default: Some(25),
            },
        );

        let mut quick_conditions = HashMap::new();
        quick_conditions.insert("words_typed".to_string(), ConditionDefinition::Fixed(25));

        let quick_practice_mode = ModeConfig {
            name: "quick_practice".to_string(),
            description: "Short practice session with word count goal".to_string(),
            parameters: quick_parameters,
            conditions: quick_conditions,
            source_overrides: HashMap::new(),
            allowed_sources: Some(vec!["random_words".to_string()]),
        };
        self.modes
            .insert("quick_practice".to_string(), quick_practice_mode);

        // Built-in timed challenge mode
        let mut timed_parameters = HashMap::new();
        timed_parameters.insert(
            "text_processing".to_string(),
            crate::sources::ParameterDefinition::Selection {
                options: vec!["normal".to_string(), "no_punctuation".to_string()],
                default: "normal".to_string(),
            },
        );

        let mut timed_conditions = HashMap::new();
        timed_conditions.insert(
            "time".to_string(),
            ConditionDefinition::Range {
                min: 30,
                max: Some(1800),
                step: Some(30),
                default: Some(300),
            },
        );

        let timed_challenge_mode = ModeConfig {
            name: "timed_challenge".to_string(),
            description: "Race against the clock".to_string(),
            parameters: timed_parameters,
            conditions: timed_conditions,
            source_overrides: HashMap::new(),
            allowed_sources: None,
        };
        self.modes
            .insert("timed_challenge".to_string(), timed_challenge_mode);
    }
}

impl ModeConfig {
    pub fn resolve_source_overrides(
        &self,
        source_name: &str,
        mode_params: &ParameterValues,
    ) -> ParameterValues {
        let mut result = ParameterValues::new();

        if let Some(overrides) = self.source_overrides.get(source_name) {
            for (param_name, template) in overrides {
                if let Ok(resolved) = self.substitute_template(template, mode_params) {
                    result.set_string(param_name.clone(), resolved);
                }
            }
        }

        result
    }

    fn substitute_template(
        &self,
        template: &str,
        params: &ParameterValues,
    ) -> Result<String, crate::sources::TemplateError> {
        let pattern = regex::Regex::new(r"\{\{(\w+)\}\}").map_err(|_| {
            crate::sources::TemplateError::InvalidSyntax("Invalid regex pattern".to_string())
        })?;
        let mut result = template.to_string();

        for cap in pattern.captures_iter(template) {
            let var_name = &cap[1];
            let value = params.get_as_string(var_name).ok_or_else(|| {
                crate::sources::TemplateError::UndefinedVariable(var_name.to_string())
            })?;
            result = result.replace(&cap[0], &value);
        }

        Ok(result)
    }
}
