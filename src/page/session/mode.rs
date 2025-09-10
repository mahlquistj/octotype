use std::{
    io::Read,
    process::{Child, Command},
};

use derive_more::From;
use thiserror::Error;

use crate::config::source::OutputFormat;

pub struct SessionMode {
    name: String,
    conditions: Conditions,
    source: Source,
}

pub struct Conditions {
    time: u64,
    words_typed: usize,
}

pub struct Source {
    command: Command,
    error_handler: (),
    child: Option<Child>,
    format: OutputFormat,
}

#[derive(Debug, Error, From)]
pub enum FetchError {
    #[error("I/O Error: {0}")]
    IO(std::io::Error),

    #[error("Encountered error: {0}")]
    SourceError(String),
}

impl Source {
    pub fn fetch(&mut self) -> Result<Option<Vec<String>>, FetchError> {
        let Some(child) = self.child.as_mut() else {
            self.child = Some(self.command.spawn()?);
            return Ok(None);
        };

        let Some(status) = child.try_wait()? else {
            return Ok(None);
        };

        if !status.success() {
            return Err(FetchError::SourceError(format!(
                "Source process returned bad exit code: {status}"
            )));
        }

        let Some(mut stdout) = child.stdout.take() else {
            return Err(FetchError::SourceError(
                "Source output was empty".to_string(),
            ));
        };

        let mut output = String::new();
        stdout.read_to_string(&mut output)?;

        return Ok(parse_output(output, &self.format));
    }
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
