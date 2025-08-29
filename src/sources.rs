use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use thiserror::Error;

use crate::page::session::EmptySessionError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSource {
    pub name: String,
    pub command: Vec<String>,
    pub timeout_seconds: u64,
    pub output_format: OutputFormat,
    pub description: Option<String>,
    pub args_template: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    JsonArray,      // ["word1", "word2", "word3"]
    Lines,          // "word1\nword2\nword3\n"
    SpaceSeparated, // "word1 word2 word3"
}

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
    ParseError(String),

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
}

pub type SourceResult<T> = Result<T, SourceError>;

// Generic arguments that can be passed to any source
#[derive(Debug, Clone, Default)]
pub struct SourceArgs {
    pub word_count: Option<usize>,
    pub max_length: Option<usize>,
    pub difficulty: Option<String>,
    pub text_processing: Option<String>,
    pub custom_params: HashMap<String, String>,
}

impl ExternalSource {
    pub fn fetch(&self, args: &SourceArgs) -> SourceResult<Vec<String>> {
        let mut cmd = Command::new(&self.command[0]);
        cmd.args(&self.command[1..]);

        // Apply template substitutions
        for (key, template) in &self.args_template {
            let value = self.substitute_template(template, args);
            cmd.arg(format!("--{}", key)).arg(value);
        }

        // Execute with timeout
        let output = cmd.output().map_err(|e| SourceError::ExternalCommand {
            command: self.command[0].clone(),
            error: e,
        })?;

        if !output.status.success() {
            return Err(SourceError::ExternalCommandFailed {
                command: self.command[0].clone(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        self.parse_output(&output.stdout)
    }

    fn substitute_template(&self, template: &str, args: &SourceArgs) -> String {
        template
            .replace("{word_count}", &args.word_count.unwrap_or(50).to_string())
            .replace("{max_length}", &args.max_length.unwrap_or(10).to_string())
            .replace(
                "{difficulty}",
                args.difficulty.as_deref().unwrap_or("medium"),
            )
            .replace(
                "{text_processing}",
                args.text_processing.as_deref().unwrap_or("normal"),
            )
    }

    fn parse_output(&self, output: &[u8]) -> SourceResult<Vec<String>> {
        let content = String::from_utf8_lossy(output);

        if content.trim().is_empty() {
            return Err(SourceError::EmptyOutput);
        }

        let words = match self.output_format {
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
    sources: HashMap<String, ExternalSource>,
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
        let quotes_source = ExternalSource {
            name: "quotes".to_string(),
            command: vec![
                "bash".to_string(), 
                "-c".to_string(), 
                r#"quote=$(curl -s --max-time 5 'https://api.quotable.io/random' | jq -r '.content' 2>/dev/null || echo "The quick brown fox jumps over the lazy dog"); echo "$quote" | tr ' ' '\n'"#.to_string()
            ],
            timeout_seconds: 10,
            output_format: OutputFormat::Lines,
            description: Some("Inspirational quotes from quotable.io with fallback".to_string()),
            args_template: HashMap::new(),
        };
        self.sources.insert("quotes".to_string(), quotes_source);

        // Built-in random words source
        let mut random_words_template = HashMap::new();
        random_words_template.insert("word-count".to_string(), "{word_count}".to_string());
        random_words_template.insert("max-length".to_string(), "{max_length}".to_string());

        let random_words_source = ExternalSource {
            name: "random_words".to_string(),
            command: vec![
                "bash".to_string(),
                "-c".to_string(),
                r#"count=${1:-50}; max_length=${2:-15}; if [ -f "/usr/share/dict/words" ]; then dict="/usr/share/dict/words"; elif [ -f "/usr/dict/words" ]; then dict="/usr/dict/words"; else echo -e "the\nquick\nbrown\nfox\njumps\nover\nlazy\ndog\npack\nmy\nbox\nwith\nfive\ndozen\nliquor\njugs"; exit 0; fi; awk "length <= $max_length" "$dict" | shuf | head -n "$count""#.to_string()
            ],
            timeout_seconds: 5,
            output_format: OutputFormat::Lines,
            description: Some("Random words from system dictionary".to_string()),
            args_template: random_words_template,
        };
        self.sources
            .insert("random_words".to_string(), random_words_source);
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
                let config: SourceConfig =
                    toml::from_str(&content).map_err(|e| SourceError::ParseError(e.to_string()))?;
                manager
                    .sources
                    .insert(config.source.name.clone(), config.source);
            }
        }

        Ok(manager)
    }

    pub fn get_source(&self, name: &str) -> Option<&ExternalSource> {
        self.sources.get(name)
    }

    pub fn list_sources(&self) -> Vec<&str> {
        self.sources.keys().map(|s| s.as_str()).collect()
    }
}

#[derive(Deserialize)]
struct SourceConfig {
    source: ExternalSource,
}

// Temporary compatibility layer for existing menu system
#[derive(Debug, Clone, strum::VariantNames)]
pub enum Source {
    DefaultWords,
}

impl Default for Source {
    fn default() -> Self {
        Self::DefaultWords
    }
}

impl Source {
    pub fn fetch(self, _args: Args) -> SourceResult<Vec<String>> {
        // Provide some default words for now
        Ok(vec![
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
            "jumps".to_string(),
            "over".to_string(),
            "lazy".to_string(),
            "dog".to_string(),
            "pack".to_string(),
            "my".to_string(),
            "box".to_string(),
            "with".to_string(),
            "five".to_string(),
            "dozen".to_string(),
            "liquor".to_string(),
            "jugs".to_string(),
        ])
    }

    pub fn get_default_args(&self) -> Args {
        Args::default()
    }
}

impl From<Source> for &'static str {
    fn from(source: Source) -> Self {
        match source {
            Source::DefaultWords => "DefaultWords",
        }
    }
}

impl std::str::FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DefaultWords" => Ok(Self::DefaultWords),
            _ => Err(format!("Unknown source: {}", s)),
        }
    }
}

// Temporary compatibility Args type
#[derive(Debug, Default)]
pub struct Args(Vec<(String, ArgValue)>);

impl Args {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, ArgValue)> {
        self.0.iter()
    }
}

impl std::ops::Index<usize> for Args {
    type Output = (String, ArgValue);

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Args {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Debug)]
pub struct ArgValue {
    value: String,
}

impl ArgValue {
    pub const fn new(value: String) -> Self {
        Self { value }
    }

    pub fn render(&self) -> Vec<ratatui::text::Span<'static>> {
        vec![ratatui::text::Span::raw(self.value.clone())]
    }

    pub const fn update(&mut self, event: &crossterm::event::KeyEvent) {
        // Placeholder - actual implementation would handle key events
        // For now, just ignore the event to satisfy the type checker
        let _ = event;
    }
}

impl Default for ArgValue {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
