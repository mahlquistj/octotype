use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Style, Stylize},
    text::Span,
};
use serde::Deserialize;
use strum::{EnumString, IntoStaticStr, VariantNames};

use crate::app::session::EmptySessionError;

mod quote_api;
mod random_words;

pub type Args = Vec<(&'static str, Box<dyn SettingValue + Send>)>;

/// Errors from word sources
#[derive(Debug)]
pub enum SourceError {
    IO(std::io::Error),
    Request {
        error: minreq::Error,
        content: Option<String>,
    },
    MissingArg(String),
    EmptySession,
}

impl std::error::Error for SourceError {}

impl From<std::io::Error> for SourceError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<minreq::Error> for SourceError {
    fn from(value: minreq::Error) -> Self {
        Self::Request {
            error: value,
            content: None,
        }
    }
}

impl From<EmptySessionError> for SourceError {
    fn from(_value: EmptySessionError) -> Self {
        Self::EmptySession
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::Request { error, content } => content.as_ref().map_or_else(
                || format!("Request error: {error}"),
                |c| format!("Request error: {error}\nContents: {c}"),
            ),
            Self::IO(e) => format!("Request error: {e}"),
            Self::MissingArg(arg) => format!("Missing argument: {arg}"),
            Self::EmptySession => "Source returned 0 words..".to_string(),
        };

        write!(f, "{error}")
    }
}

/// The different souces we get words from
#[derive(VariantNames, IntoStaticStr, EnumString, Clone, Debug)]
pub enum Source {
    // TODO: CommonWords,
    RandomWords,
    Quote,
}

impl Default for Source {
    fn default() -> Self {
        Self::Quote
    }
}

impl Source {
    pub fn get_default_args(&self) -> Args {
        match self {
            Self::Quote => Args::new(),
            Self::RandomWords => {
                vec![
                    ("number", Box::new(15usize)),
                    ("length", Box::new(None::<usize>)),
                ]
            }
        }
    }

    pub const fn uses_request(&self) -> bool {
        match self {
            Self::Quote | Self::RandomWords => true,
            // Leaving room for future implementations
        }
    }

    pub const fn get_path(&self) -> &str {
        match self {
            Self::Quote => "https://api.quotable.kurokeita.dev/api/quotes/random",
            Self::RandomWords => "https://random-word-api.herokuapp.com/word",
        }
    }

    fn parse_response_json<'d, T: Deserialize<'d>>(
        response: &'d minreq::Response,
    ) -> Result<T, SourceError> {
        match response.json::<T>() {
            Ok(json) => Ok(json),
            Err(error) => Err(SourceError::Request {
                error,
                content: response.as_str().map(str::to_string).ok(),
            }),
        }
    }

    pub fn parse_response(&self, response: minreq::Response) -> Result<Vec<String>, SourceError> {
        let parsed = match self {
            Self::Quote => Self::parse_response_json::<quote_api::QuoteWrapper>(&response)?
                .quote
                .content
                .split_ascii_whitespace()
                .map(str::to_string)
                .collect::<Vec<String>>(),
            Self::RandomWords => Self::parse_response_json(&response)?,
        };

        Ok(parsed)
    }

    /// Fetch words from the source
    pub fn fetch(&self, args: Args) -> Result<Vec<String>, SourceError> {
        if self.uses_request() {
            let url = self.get_path();
            return self.fetch_request(url, args);
        }

        unreachable!("No method found for source {self:?}")
    }

    fn fetch_request(&self, url: &str, args: Args) -> Result<Vec<String>, SourceError> {
        let mut req = minreq::get(url);

        for (key, value) in args {
            let Some(value) = value.get_selected() else {
                continue;
            };
            req = req.with_param(key, &value);
        }

        let result = req.send()?;

        self.parse_response(result)
    }
}

pub enum SettingEvent {
    // Useful for numbers and lists
    Increment,
    Decrement,

    // Useful for strings
    Input(char),
    RemoveInput,

    Clear,
}

impl<'event> TryFrom<&'event KeyEvent> for SettingEvent {
    type Error = ();

    fn try_from(value: &'event KeyEvent) -> Result<Self, Self::Error> {
        let setting_event = match value.code {
            KeyCode::Right => Self::Increment,
            KeyCode::Left => Self::Decrement,
            KeyCode::Char(c) => Self::Input(c),
            KeyCode::Backspace => Self::RemoveInput,
            KeyCode::Delete => Self::Clear,
            _ => return Err(()),
        };

        Ok(setting_event)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
// Allowing dead code for now, as this might be used for lists or ranges in the future
pub struct SourceSetting<V, S> {
    values: V,
    selected: S,
}

impl<V: Default, S: Default> Default for SourceSetting<V, S> {
    fn default() -> Self {
        Self {
            values: V::default(),
            selected: S::default(),
        }
    }
}

pub trait SettingValue {
    fn update(&mut self, event: &SettingEvent);
    fn render(&self) -> Vec<Span>;
    fn get_selected(&self) -> Option<String>;
}

impl SettingValue for String {
    fn update(&mut self, event: &SettingEvent) {
        match event {
            SettingEvent::Input(ch) => self.push(*ch),
            SettingEvent::RemoveInput => {
                self.pop();
            }
            _ => (),
        }
    }

    fn render(&self) -> Vec<Span> {
        let len = self.len().saturating_sub(1);

        let mut text_vec = vec![];

        if len > 0 {
            text_vec.push(Span::raw(&self[0..(len)])); // Get everything up to the second last
        }

        text_vec.push(Span::styled(" ", Style::default().reversed()));

        text_vec
    }

    fn get_selected(&self) -> Option<String> {
        Some(self.clone())
    }
}

// TODO: Temporary: Find a better solution for optional values
impl<S: SettingValue + Default> SettingValue for Option<S> {
    fn update(&mut self, event: &SettingEvent) {
        if matches!(*event, SettingEvent::Clear) {
            *self = None;
            return;
        }

        let mut setting = self.take().unwrap_or_default();

        setting.update(event);

        *self = Some(setting);
    }

    fn render(&self) -> Vec<Span> {
        if let Some(setting) = &self {
            return setting.render();
        }

        vec![Span::raw("-")]
    }

    fn get_selected(&self) -> Option<String> {
        self.as_ref().and_then(SettingValue::get_selected)
    }
}

macro_rules! impl_number {
    ($number:ty) => {
        impl SettingValue for $number {
            fn update(&mut self, event: &SettingEvent) {
                match event {
                    SettingEvent::Increment => *self += 1,
                    SettingEvent::Decrement => *self -= 1,
                    _ => (),
                }
            }

            fn render(&self) -> Vec<Span> {
                vec![Span::raw(self.to_string())]
            }

            fn get_selected(&self) -> Option<String> {
                Some(self.to_string())
            }
        }
    };
    ($($number:ty),*) => {
        $(impl_number!($number);)*
    };
}

impl_number!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);
