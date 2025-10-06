use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    string::FromUtf8Error,
    time::Duration,
};

use derive_more::From;
use thiserror::Error;

use crate::config::{
    ModeConfig, SourceConfig,
    mode::{ConditionConfig, ParseConditionError},
    parameters::ParameterValues,
    source::Formatting,
};

#[derive(Debug, Error, From)]
pub enum CreateModeError {
    #[error("Condition{0}")]
    Condition(ParseConditionError),

    #[error("Unable to find '{tool}' in path: {error}")]
    ToolMissing { tool: String, error: which::Error },
}

#[derive(Debug)]
pub struct Mode {
    pub conditions: Conditions,
    pub source: Source,
}

impl Mode {
    pub fn from_config(
        sources_dir: PathBuf,
        mode: ModeConfig,
        source: SourceConfig,
        parameters: ParameterValues,
    ) -> Result<Self, CreateModeError> {
        let resolved_conditions = Conditions::from_config(mode.conditions, &parameters)?;
        let resolved_source = Source::from_config(sources_dir, source, &parameters)?;
        Ok(Self {
            conditions: resolved_conditions,
            source: resolved_source,
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
        config: ConditionConfig,
        parameters: &ParameterValues,
    ) -> Result<Self, CreateModeError> {
        let time = config
            .time
            .map(|value| {
                value
                    .parse_number("time", parameters)
                    .map(|secs| Duration::from_secs(secs as u64))
            })
            .transpose()?;

        let words_typed = config
            .words_typed
            .map(|value| value.parse_number("words_typed", parameters))
            .transpose()?;

        let allow_deletions = config
            .allow_deletions
            .parse_bool("allow_deletions", parameters)?;

        let allow_errors = config.allow_errors.parse_bool("allow_errors", parameters)?;

        Ok(Self {
            time,
            words_typed,
            allow_deletions,
            allow_errors,
        })
    }
}

#[derive(Debug)]
pub struct Source {
    command: Command,
    child: Option<Child>,
    format: Formatting,
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
        // Take child process out
        let Some(mut child) = self.child.take() else {
            self.child = Some(self.command.spawn()?);
            return Ok(None);
        };

        let Some(status) = child.try_wait()? else {
            // Put child process back
            self.child = Some(child);
            return Ok(None);
        };

        let process_output = child.wait_with_output()?;

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

        Ok(parse_output(stdout, &self.format))
    }

    pub fn from_config(
        sources_dir: PathBuf,
        config: SourceConfig,
        parameters: &ParameterValues,
    ) -> Result<Self, CreateModeError> {
        // Ensure required tools exist in path
        config
            .meta
            .required_tools
            .into_iter()
            .try_for_each(|tool| {
                which::which(&tool)
                    .map(|_| ())
                    .map_err(|error| (tool, error))
            })?;

        let mut program = config
            .meta
            .command
            .iter()
            .map(|string| parameters.replace_values(string))
            .collect::<Vec<String>>();

        let mut command = std::process::Command::new(program.remove(0));
        command
            .args(program)
            .current_dir(sources_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        Ok(Self {
            command,
            child: None,
            format: config.meta.formatting,
        })
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
