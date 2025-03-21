use std::fmt::Display;

use strum::{EnumCount, EnumIter, EnumString, IntoStaticStr, VariantNames};

mod quote_api;
mod random_words;

/// Errors from word sources
#[derive(Debug)]
pub enum SourceError {
    IO(std::io::Error),
    Request(minreq::Error),
}

impl std::error::Error for SourceError {}

impl From<std::io::Error> for SourceError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<minreq::Error> for SourceError {
    fn from(value: minreq::Error) -> Self {
        Self::Request(value)
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::Request(e) => format!("Request error: {e}"),
            Self::IO(e) => format!("Request error: {e}"),
        };

        write!(f, "{error}")
    }
}

/// The different souces we get words from
#[derive(VariantNames, IntoStaticStr, EnumString, Clone)]
pub enum Source {
    CommonWords {
        // The language selected
        lang: String,
    },

    RandomWords {
        // The language selected (None = English)
        lang: Option<String>,
    },

    Quote,
}

impl Default for Source {
    fn default() -> Self {
        Self::Quote
    }
}

impl Source {
    /// Fetch words from the source
    pub fn fetch(&self, amount: u32, max_length: Option<u32>) -> Result<Vec<String>, SourceError> {
        let words = match self {
            Self::CommonWords { .. } => todo!("Implement commonwords"),
            Self::RandomWords { lang } => {
                random_words::get_words(lang.as_ref(), amount, max_length)?
            }
            Self::Quote => quote_api::get_words()?,
        };

        Ok(words)
    }
}
