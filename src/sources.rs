use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;

use crate::page::session::EmptySessionError;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("External command '{command}' failed to execute: {error}")]
    ExternalCommand {
        command: String,
        error: std::io::Error,
    },

    #[error("External command '{command}' failed with exit code {exit_code:?}: {stderr}")]
    ExternalCommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Command timeout after {timeout_seconds} seconds")]
    Timeout { timeout_seconds: u64 },

    #[error("Failed to parse command output: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("No words returned from source")]
    EmptyOutput,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Session creation error: {0}")]
    SessionCreation(String),

    #[error("Empty session error")]
    EmptySession(#[from] EmptySessionError),

    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Parameter validation error: {0}")]
    ParameterValidation(String),
}

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Undefined variable '{0}' in template")]
    UndefinedVariable(String),

    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub meta: SourceMeta,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,
    #[serde(default)]
    pub error_handling: SourceErrorHandling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub name: String,
    pub description: String,
    pub command: Vec<String>,
    pub timeout_seconds: u64,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterDefinition {
    Range {
        min: i32,
        max: i32,
        step: i32,
        #[serde(default)]
        default: Option<i32>,
    },
    Selection {
        options: Vec<String>,
        default: String,
    },
    Toggle(bool), // Simple boolean with default value
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SourceErrorHandling {
    #[serde(default)]
    required_tools: Vec<String>,
    #[serde(default)]
    network_required: bool,
    #[serde(default)]
    max_retries: i8,
    #[serde(default)]
    offline_alternative: Option<String>,
    #[serde(default)]
    retry_delay_seconds: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    JsonArray,      // ["word1", "word2", "word3"]
    Lines,          // "word1\nword2\nword3\n"
    SpaceSeparated, // "word1 word2 word3"
}

pub type SourceResult<T> = Result<T, SourceError>;

#[derive(Debug, Clone, Default)]
pub struct ParameterValues {
    values: HashMap<String, ParameterValue>,
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
    Integer(i32),
    String(String),
    Boolean(bool),
}

impl ParameterValues {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set_integer(&mut self, key: String, value: i32) {
        self.values.insert(key, ParameterValue::Integer(value));
    }

    pub fn set_string(&mut self, key: String, value: String) {
        self.values.insert(key, ParameterValue::String(value));
    }

    pub fn set_boolean(&mut self, key: String, value: bool) {
        self.values.insert(key, ParameterValue::Boolean(value));
    }

    pub fn get_integer(&self, key: &str) -> Option<i32> {
        match self.values.get(key)? {
            ParameterValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.values.get(key)? {
            ParameterValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.values.get(key)? {
            ParameterValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn get_as_string(&self, key: &str) -> Option<String> {
        match self.values.get(key)? {
            ParameterValue::Integer(i) => Some(i.to_string()),
            ParameterValue::String(s) => Some(s.clone()),
            ParameterValue::Boolean(b) => Some(b.to_string()),
        }
    }
}

// For backward compatibility with existing code
#[derive(Debug, Clone, Default)]
pub struct SourceArgs(HashMap<String, String>);

impl From<&ParameterValues> for SourceArgs {
    fn from(params: &ParameterValues) -> Self {
        let mut args = HashMap::new();
        for (key, value) in &params.values {
            let str_value = match value {
                ParameterValue::Integer(i) => i.to_string(),
                ParameterValue::String(s) => s.clone(),
                ParameterValue::Boolean(b) => b.to_string(),
            };
            args.insert(key.clone(), str_value);
        }
        SourceArgs(args)
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    config: SourceConfig,
}

impl Source {
    pub fn new(config: SourceConfig) -> Self {
        Self { config }
    }

    pub fn name(&self) -> &str {
        &self.config.meta.name
    }

    pub fn description(&self) -> &str {
        &self.config.meta.description
    }

    pub fn get_parameter_definitions(&self) -> &HashMap<String, ParameterDefinition> {
        &self.config.parameters
    }

    pub fn create_default_parameters(&self) -> ParameterValues {
        let mut params = ParameterValues::new();
        
        for (key, param_def) in &self.config.parameters {
            match param_def {
                ParameterDefinition::Range { min, default, .. } => {
                    let value = default.unwrap_or(*min);
                    params.set_integer(key.clone(), value);
                }
                ParameterDefinition::Selection { default, .. } => {
                    params.set_string(key.clone(), default.clone());
                }
                ParameterDefinition::Toggle(default_value) => {
                    params.set_boolean(key.clone(), *default_value);
                }
            }
        }
        
        params
    }

    pub fn validate_parameters(&self, params: &ParameterValues) -> SourceResult<()> {
        for (key, param_def) in &self.config.parameters {
            match param_def {
                ParameterDefinition::Range { min, max, .. } => {
                    if let Some(value) = params.get_integer(key) {
                        if value < *min || value > *max {
                            return Err(SourceError::ParameterValidation(
                                format!("Parameter '{}' value {} is out of range [{}, {}]", key, value, min, max)
                            ));
                        }
                    }
                }
                ParameterDefinition::Selection { options, .. } => {
                    if let Some(value) = params.get_string(key) {
                        if !options.contains(&value.to_string()) {
                            return Err(SourceError::ParameterValidation(
                                format!("Parameter '{}' value '{}' is not in allowed options: {:?}", key, value, options)
                            ));
                        }
                    }
                }
                ParameterDefinition::Toggle(_) => {
                    // Boolean parameters are always valid
                }
            }
        }
        Ok(())
    }

    pub fn fetch(&self, params: &ParameterValues) -> SourceResult<Vec<String>> {
        self.validate_parameters(params)?;
        
        let resolved_command = self.resolve_command(params)?;
        
        let mut cmd = Command::new(&resolved_command[0]);
        if resolved_command.len() > 1 {
            cmd.args(&resolved_command[1..]);
        }

        // Execute with timeout using std::process (timeout handling would require additional crates)
        let output = cmd.output().map_err(|e| SourceError::ExternalCommand {
            command: resolved_command[0].clone(),
            error: e,
        })?;

        if !output.status.success() {
            return Err(SourceError::ExternalCommandFailed {
                command: resolved_command[0].clone(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        self.parse_output(&output.stdout)
    }

    fn resolve_command(&self, params: &ParameterValues) -> SourceResult<Vec<String>> {
        let mut resolved_command = Vec::new();
        
        for part in &self.config.meta.command {
            let resolved_part = self.substitute_template(part, params)?;
            resolved_command.push(resolved_part);
        }
        
        Ok(resolved_command)
    }

    fn substitute_template(&self, template: &str, params: &ParameterValues) -> Result<String, TemplateError> {
        let pattern = Regex::new(r"\{\{(\w+)\}\}").map_err(|_| TemplateError::InvalidSyntax("Invalid regex pattern".to_string()))?;
        let mut result = template.to_string();
        
        for cap in pattern.captures_iter(template) {
            let var_name = &cap[1];
            let value = params.get_as_string(var_name)
                .ok_or_else(|| TemplateError::UndefinedVariable(var_name.to_string()))?;
            result = result.replace(&cap[0], &value);
        }
        
        Ok(result)
    }

    fn parse_output(&self, output: &[u8]) -> SourceResult<Vec<String>> {
        let content = String::from_utf8_lossy(output);

        if content.trim().is_empty() {
            return Err(SourceError::EmptyOutput);
        }

        let words = match self.config.meta.output_format {
            OutputFormat::JsonArray => serde_json::from_str::<Vec<String>>(&content)?,
            OutputFormat::Lines => content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            OutputFormat::SpaceSeparated => {
                content.split_whitespace().map(|s| s.to_string()).collect()
            }
        };

        if words.is_empty() {
            Err(SourceError::EmptyOutput)
        } else {
            Ok(words)
        }
    }
}

// Source discovery and management
pub struct SourceManager {
    sources: HashMap<String, Source>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.add_builtin_sources();
        manager
    }

    fn add_builtin_sources(&mut self) {
        // Built-in quotes source
        let quotes_config = SourceConfig {
            meta: SourceMeta {
                name: "quotes".to_string(),
                description: "Inspirational quotes from quotable.io with fallback".to_string(),
                command: vec![
                    "bash".to_string(), 
                    "-c".to_string(), 
                    r#"quote=$(curl -s --max-time 5 'https://api.quotable.io/random' | jq -r '.content' 2>/dev/null || echo "The quick brown fox jumps over the lazy dog"); echo "$quote" | tr ' ' '\n'"#.to_string()
                ],
                timeout_seconds: 10,
                output_format: OutputFormat::Lines,
            },
            parameters: HashMap::new(),
            error_handling: SourceErrorHandling::default(),
        };
        let quotes_source = Source::new(quotes_config);
        self.sources.insert("quotes".to_string(), quotes_source);

        // Built-in random words source
        let mut random_words_parameters = HashMap::new();
        random_words_parameters.insert(
            "word_count".to_string(),
            ParameterDefinition::Range {
                min: 10,
                max: 200,
                step: 5,
                default: Some(50),
            }
        );
        random_words_parameters.insert(
            "max_length".to_string(),
            ParameterDefinition::Range {
                min: 3,
                max: 20,
                step: 1,
                default: Some(10),
            }
        );

        let random_words_config = SourceConfig {
            meta: SourceMeta {
                name: "random_words".to_string(),
                description: "Random words from system dictionary".to_string(),
                command: vec![
                    "bash".to_string(),
                    "-c".to_string(),
                    r#"count={{word_count}}; max_length={{max_length}}; if [ -f "/usr/share/dict/words" ]; then dict="/usr/share/dict/words"; elif [ -f "/usr/dict/words" ]; then dict="/usr/dict/words"; else echo -e "the\nquick\nbrown\nfox\njumps\nover\nlazy\ndog\npack\nmy\nbox\nwith\nfive\ndozen\nliquor\njugs"; exit 0; fi; awk "length <= $max_length" "$dict" | shuf | head -n "$count""#.to_string()
                ],
                timeout_seconds: 5,
                output_format: OutputFormat::Lines,
            },
            parameters: random_words_parameters,
            error_handling: SourceErrorHandling::default(),
        };
        let random_words_source = Source::new(random_words_config);
        self.sources.insert("random_words".to_string(), random_words_source);
    }

    pub fn load_from_config_dir(config_dir: &std::path::Path) -> SourceResult<Self> {
        let mut manager = Self::with_defaults();

        let sources_dir = config_dir.join("sources");
        if !sources_dir.exists() {
            return Ok(manager);
        }

        for entry in std::fs::read_dir(&sources_dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                let content = std::fs::read_to_string(&path)?;
                let source_config: SourceConfig = toml::from_str(&content)?;
                let source = Source::new(source_config);
                manager.sources.insert(source.name().to_string(), source);
            }
        }

        Ok(manager)
    }

    pub fn get_source(&self, name: &str) -> Option<&Source> {
        self.sources.get(name)
    }

    pub fn list_sources(&self) -> Vec<&str> {
        self.sources.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_sources(&self) -> &HashMap<String, Source> {
        &self.sources
    }
}
