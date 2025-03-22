use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Style, Stylize},
    text::Span,
};
use strum::{EnumString, IntoStaticStr, VariantNames};

mod quote_api;
mod random_words;

pub type Args = Vec<(&'static str, Box<dyn SettingValue + Send>)>;

/// Errors from word sources
#[derive(Debug)]
pub enum SourceError {
    IO(std::io::Error),
    Request(minreq::Error),
    MissingArg(String),
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
            Self::MissingArg(arg) => format!("Missing argument: {arg}"),
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
                    (
                        "length",
                        Box::new(SourceSetting {
                            values: (),
                            selected: 80usize,
                        }),
                    ),
                    (
                        "amount",
                        Box::new(SourceSetting {
                            values: (),
                            selected: 15usize,
                        }),
                    ),
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

    pub fn parse_response(&self, response: minreq::Response) -> Result<Vec<String>, SourceError> {
        let parsed = match self {
            Self::Quote => response
                .json::<quote_api::QuoteWrapper>()?
                .quote
                .content
                .split_ascii_whitespace()
                .map(str::to_string)
                .collect::<Vec<String>>(),
            Self::RandomWords => response.json()?,
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
            let value = value.get_selected();
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
}

impl<'event> TryFrom<&'event KeyEvent> for SettingEvent {
    type Error = ();

    fn try_from(value: &'event KeyEvent) -> Result<Self, Self::Error> {
        let setting_event = match value.code {
            KeyCode::Right => Self::Increment,
            KeyCode::Left => Self::Decrement,
            KeyCode::Char(c) => Self::Input(c),
            KeyCode::Backspace => Self::RemoveInput,
            _ => return Err(()),
        };

        Ok(setting_event)
    }
}

#[derive(Clone)]
pub struct SourceSetting<V, S> {
    #[allow(dead_code)] // Allowing dead code for now, as this might be used in the future
    values: V,
    selected: S,
}

pub trait SettingValue {
    fn update(&mut self, event: &SettingEvent);
    fn render(&self) -> Vec<Span>;
    fn get_selected(&self) -> String;
}

impl SettingValue for SourceSetting<(), String> {
    fn update(&mut self, event: &SettingEvent) {
        match event {
            SettingEvent::Input(ch) => self.selected.push(*ch),
            SettingEvent::RemoveInput => {
                self.selected.pop();
            }
            _ => (),
        }
    }

    fn render(&self) -> Vec<Span> {
        let len = self.selected.len().saturating_sub(1);

        let mut text_vec = vec![];

        if len > 0 {
            text_vec.push(Span::raw(&self.selected[0..(len)])); // Get everything up to the second last
        }

        text_vec.push(Span::styled(" ", Style::default().reversed()));

        text_vec
    }

    fn get_selected(&self) -> String {
        self.selected.clone()
    }
}

macro_rules! impl_number {
    ($number:ty) => {
        impl SettingValue for SourceSetting<(), $number> {
            fn update(&mut self, event: &SettingEvent) {
                match event {
                    SettingEvent::Increment => self.selected += 1 as $number,
                    SettingEvent::Decrement => self.selected -= 1 as $number,
                    _ => (),
                }
            }

            fn render(&self) -> Vec<Span> {
                vec![Span::raw(self.selected.to_string())]
            }

            fn get_selected(&self) -> String {
                self.selected.to_string()
            }
        }
    };
    ($($number:ty),*) => {
        $(impl_number!($number);)*
    };
}

impl_number!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, usize, isize);
