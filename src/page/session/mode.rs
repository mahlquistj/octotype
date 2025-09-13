use std::{
    num::ParseIntError,
    path::PathBuf,
    process::{Child, Command, Stdio},
    string::FromUtf8Error,
    sync::LazyLock,
    time::Duration,
};

use derive_more::From;
use regex::Regex;
use thiserror::Error;

pub static RE_HANDLEBARS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{(.+?)\}").unwrap());

use crate::config::{
    ModeConfig, SourceConfig,
    mode::{ConditionConfig, ConditionValue},
    parameters::ParameterValues,
    source::OutputFormat,
};

#[derive(Debug, Error)]
#[error("Failed to create mode: {error}\n{mode_config:?}\n{source_config:?}\n{parameters:?}")]
pub struct CreateModeError {
    error: ParseIntError,
    mode_config: ModeConfig,
    source_config: SourceConfig,
    parameters: ParameterValues,
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
        let resolved_conditions = match Conditions::from_config(&mode.conditions, &parameters) {
            Ok(conditions) => conditions,
            Err(error) => {
                return Err(CreateModeError {
                    error,
                    mode_config: mode,
                    source_config: source,
                    parameters,
                });
            }
        };
        let resolved_source = Source::from_config(sources_dir, source, &parameters);
        Ok(Self {
            conditions: resolved_conditions,
            source: resolved_source,
        })
    }
}

#[derive(Debug)]
pub struct Conditions {
    pub time: Option<Duration>,
    pub words_typed: Option<i64>,
}

impl Conditions {
    pub fn from_config(
        config: &ConditionConfig,
        parameters: &ParameterValues,
    ) -> Result<Self, ParseIntError> {
        let time = if let Some(value) = &config.time {
            let secs = match value {
                ConditionValue::String(string) => replace_parameters(string, parameters).parse()?,
                ConditionValue::Number(num) => *num as u64,
                ConditionValue::Bool(_) => unreachable!("TIME WAS BOOLEAN"),
            };
            Some(Duration::from_secs(secs))
        } else {
            None
        };

        let words_typed = if let Some(value) = &config.words_typed {
            let words = match value {
                ConditionValue::String(string) => replace_parameters(string, parameters).parse()?,
                ConditionValue::Number(num) => *num,
                ConditionValue::Bool(_) => unreachable!("WORDS WAS BOOLEAN"),
            };
            Some(words)
        } else {
            None
        };

        Ok(Self { time, words_typed })
    }
}

#[derive(Debug)]
pub struct Source {
    command: Command,
    child: Option<Child>,
    format: OutputFormat,
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
    pub fn fetch(&mut self) -> Result<Vec<String>, FetchError> {
        loop {
            if let Some(words) = self.try_fetch()? {
                return Ok(words);
            }
        }
    }

    pub fn try_fetch(&mut self) -> Result<Option<Vec<String>>, FetchError> {
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
    ) -> Self {
        let mut program = config
            .meta
            .command
            .iter()
            .map(|string| replace_parameters(string, parameters))
            .collect::<Vec<String>>();

        let mut command = std::process::Command::new(program.remove(0));
        command
            .args(program)
            .current_dir(sources_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        Self {
            command,
            child: None,
            format: config.meta.output,
        }
    }
}

fn replace_parameters(string: &str, parameters: &ParameterValues) -> String {
    RE_HANDLEBARS
        .replace_all(string, |caps: &regex::Captures| {
            let Some(key) = caps.get(1).map(|m| m.as_str()) else {
                return caps.get(0).unwrap().as_str().to_string();
            };

            let Some(param) = parameters.get(key) else {
                return caps.get(0).unwrap().as_str().to_string();
            };

            param.get_value()
        })
        .to_string()
}

fn parse_output(output: String, format: &OutputFormat) -> Option<Vec<String>> {
    let words: Vec<String> = match format {
        OutputFormat::Default => output
            .split_ascii_whitespace()
            .map(str::to_string)
            .collect(),
        OutputFormat::Array => output
            .strip_prefix('[')
            .and_then(|rem| rem.strip_suffix(']'))
            .map(|words| words.split(',').map(str::to_string))?
            .collect(),
    };
    Some(words)
}

#[cfg(test)]
mod test {
    // #[test]
    // fn regex_replacement() {
    //     let mut parameters = ParameterValues::new();
    // }
}
