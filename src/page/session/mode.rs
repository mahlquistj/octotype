use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::{Child, Command, Stdio},
    string::FromUtf8Error,
    time::Duration,
};

use derive_more::From;
use rand::{rng, seq::SliceRandom};
use thiserror::Error;

use crate::config::{
    Config, ModeConfig, SourceConfig,
    mode::{ConditionConfig, ParseConditionError},
    parameters::ParameterValues,
    source::{Formatting, GeneratorDefinition, ListSource},
};

#[derive(Debug, Error, From)]
pub enum CreateModeError {
    #[error("Condition{0}")]
    Condition(ParseConditionError),

    #[error("Unable to find '{tool}' in path: {error}")]
    ToolMissing { tool: String, error: which::Error },

    #[error("Failed parsing file '{path}': {error}")]
    ParseFile {
        error: std::io::Error,
        path: PathBuf,
    },
}

#[derive(Debug)]
pub struct Mode {
    pub conditions: Conditions,
    pub source: Source,
    pub mode_name: String,
    pub source_name: String,
}

impl Mode {
    pub fn from_config(
        config: &Config,
        mode: ModeConfig,
        source: SourceConfig,
        parameters: ParameterValues,
    ) -> Result<Self, CreateModeError> {
        let mode_name = mode.meta.name.clone();
        let source_name = source.meta.name.clone();
        let resolved_conditions = Conditions::from_config(mode.conditions, &parameters)?;
        let resolved_source = Source::from_config(config, source, &parameters)?;
        Ok(Self {
            conditions: resolved_conditions,
            source: resolved_source,
            mode_name,
            source_name,
        })
    }
}

#[derive(Debug)]
pub struct Conditions {
    pub time: Option<Duration>,
    pub words_typed: Option<usize>,
    pub allow_deletions: bool,
    pub allow_errors: bool,
}

impl Conditions {
    pub fn from_config(
        condition_config: ConditionConfig,
        parameters: &ParameterValues,
    ) -> Result<Self, CreateModeError> {
        let ConditionConfig {
            time,
            words_typed,
            allow_deletions,
            allow_errors,
        } = condition_config;

        let time = time
            .map(|value| {
                value
                    .parse_number("time", parameters)
                    .map(|secs| Duration::from_secs(secs as u64))
            })
            .transpose()?;

        let words_typed = words_typed
            .map(|value| value.parse_number("words_typed", parameters))
            .transpose()?;

        let allow_deletions = allow_deletions.parse_bool("allow_deletions", parameters)?;

        let allow_errors = allow_errors.parse_bool("allow_errors", parameters)?;

        Ok(Self {
            time,
            words_typed,
            allow_deletions,
            allow_errors,
        })
    }
}

#[derive(Debug)]
pub enum Source {
    Command {
        command: Command,
        child: Option<Box<Child>>,
        format: Formatting,
    },
    List {
        words: Vec<String>,
        randomize: bool,
    },
}

#[derive(Debug, Error, From)]
pub enum FetchError {
    #[error("Fetch I/O Error: {0}")]
    IO(std::io::Error),

    #[error("Failed to get output of command")]
    Output(FromUtf8Error),

    #[error("Encountered error: {0}")]
    SourceError(String),
}

impl Source {
    pub fn fetch(&mut self) -> Result<String, FetchError> {
        loop {
            if let Some(words) = self.try_fetch()? {
                return Ok(words);
            }
        }
    }

    pub fn try_fetch(&mut self) -> Result<Option<String>, FetchError> {
        match self {
            Self::Command {
                command,
                child,
                format,
            } => {
                // Take child process out
                let Some(mut child_process) = child.take() else {
                    *child = Some(Box::new(command.spawn()?));
                    return Ok(None);
                };

                let Some(status) = child_process.try_wait()? else {
                    // Put child process back
                    *child = Some(child_process);
                    return Ok(None);
                };

                let process_output = child_process.wait_with_output()?;

                let stderr = String::from_utf8(process_output.stderr)?;
                let stdout = String::from_utf8(process_output.stdout)?;

                if !status.success() {
                    return Err(FetchError::SourceError(format!(
                        "Source process returned bad exit code: {status}\nStderr: {stderr}\nStdout: {stdout}"
                    )));
                }

                if stdout.is_empty() {
                    return Err(FetchError::SourceError(
                        "Source output was empty!".to_string(),
                    ));
                }

                Ok(parse_output(stdout, format))
            }
            Self::List { words, randomize } => {
                if *randomize {
                    let mut rng = rng();
                    words.shuffle(&mut rng);
                    return Ok(Some(words.join(" ")));
                }
                Ok(Some(words.join(" ")))
            }
        }
    }

    pub fn from_config(
        config: &Config,
        source_config: SourceConfig,
        parameters: &ParameterValues,
    ) -> Result<Self, CreateModeError> {
        let SourceConfig { generator, .. } = source_config;

        match generator {
            GeneratorDefinition::Command {
                command,
                formatting,
                required_tools,
                ..
            } => {
                // Ensure required tools exist in path
                required_tools.into_iter().try_for_each(|tool| {
                    which::which(&tool)
                        .map(|_| ())
                        .map_err(|error| (tool, error))
                })?;

                let mut program = command
                    .iter()
                    .map(|string| parameters.replace_values(string))
                    .collect::<Vec<String>>();

                let mut command = std::process::Command::new(program.remove(0));
                command
                    .args(program)
                    .current_dir(config.sources_dir())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                Ok(Self::Command {
                    command,
                    format: formatting,
                    child: None,
                })
            }
            GeneratorDefinition::List { source, randomize } => {
                let words = match source {
                    ListSource::Array(vec) => vec,
                    ListSource::File { path, separator } => {
                        let mut buf = String::new();

                        let mut file = File::open(path.clone()).map_err(|error| {
                            CreateModeError::ParseFile {
                                error,
                                path: path.clone(),
                            }
                        })?;

                        file.read_to_string(&mut buf)
                            .map_err(|error| CreateModeError::ParseFile { error, path })?;

                        separator.map_or_else(
                            || buf.split_ascii_whitespace().map(str::to_string).collect(),
                            |sep| buf.split(sep).map(str::to_string).collect(),
                        )
                    }
                };
                Ok(Self::List { words, randomize })
            }
        }
    }
}

fn parse_output(output: String, format: &Formatting) -> Option<String> {
    let words: String = match format {
        Formatting::Raw => output,
        Formatting::Spaced => output
            .split_ascii_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    };
    Some(words)
}
