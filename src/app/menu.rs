use std::fmt::Display;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};
use serde::Deserialize;

use crate::{
    config::Config,
    session::{Segment, TypingSession},
    utils::{center, KeyEventHelper, Message, Page},
};

use super::LoadingScreen;

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

/// Wrapper for parsing quotes from quotes-api
#[derive(Deserialize)]
struct QuoteWrapper {
    quote: Quote,
}

/// A quote object from quotes-api
#[derive(Deserialize)]
#[serde(rename = "quote")]
struct Quote {
    // id: String,
    content: String,
    // author: String,
    // slug: String,
    // length: usize,
    // tags: Vec<String>,
}

/// The different souces we get words from
#[derive(Clone)]
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
    fn fetch(&self, amount: u32, max_length: Option<u32>) -> Result<Vec<String>, SourceError> {
        let words = match self {
            Self::CommonWords { .. } => todo!("Implement commonwords"),
            Self::RandomWords { lang } => {
                let mut req = minreq::get("https://random-word-api.herokuapp.com/word")
                    .with_param("number", amount.to_string());

                if let Some(language) = lang {
                    req = req.with_param("lang", language);
                }

                if let Some(ml) = max_length {
                    req = req.with_param("length", ml.to_string());
                }

                req.send()?.json::<Vec<String>>()?
            }
            Self::Quote => minreq::get("https://api.quotable.kurokeita.dev/api/quotes/random")
                .send()?
                .json::<QuoteWrapper>()?
                .quote
                .content
                .split_ascii_whitespace()
                .map(str::to_string)
                .collect::<Vec<String>>(),
        };

        Ok(words)
    }
}

/// Page: Main menu
pub struct Menu {
    source: Source,
    words_amount: u32,
    max_word_length: u32,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            source: Source::default(),
            words_amount: 10,
            max_word_length: 0,
        }
    }
}

impl Menu {
    /// Creates a new menu
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `TypingSession` with the given parameters
    fn create_session(
        source: Source,
        words_amount: u32,
        max_word_length: Option<u32>,
    ) -> Result<TypingSession, SourceError> {
        let words = source.fetch(words_amount, max_word_length)?;
        let last_segment = words.chunks(5).count() - 1;

        let words = words
            .chunks(5)
            .enumerate()
            .map(|(idx, words)| {
                let mut string = words
                    .iter()
                    .cloned()
                    .map(|mut word| {
                        word.push(' ');
                        word
                    })
                    .collect::<String>();

                if idx == last_segment {
                    string.pop();
                }

                Segment::from_iter(string.chars())
            })
            .collect();

        Ok(TypingSession::new(words))
    }
}

impl Page for Menu {
    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        _config: &Config,
    ) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        let text = vec![
            Line::from(vec![
                Span::raw(format!("Words     : {}", self.words_amount)),
                Span::styled("↕", Style::new().bold()),
            ]),
            Line::from(vec![
                Span::raw(format!("Max length: {}", self.max_word_length)),
                Span::styled("↔", Style::new().bold()),
            ]),
        ];

        let settings = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(settings, center);
    }

    fn handle_events(
        &mut self,
        event: &crossterm::event::Event,
        _config: &Config,
    ) -> Option<Message> {
        if let Event::Key(key) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Up => self.words_amount = self.words_amount.saturating_add(1),
                    KeyCode::Down => self.words_amount = self.words_amount.saturating_sub(1),

                    KeyCode::Right => self.max_word_length = self.max_word_length.saturating_add(1),
                    KeyCode::Left => self.max_word_length = self.max_word_length.saturating_sub(1),

                    KeyCode::Enter => {
                        let source = self.source.clone();
                        let words_amount = self.words_amount;

                        let max_word_length =
                            (self.max_word_length > 0).then_some(self.max_word_length);

                        // Spawn a `LoadingScreen` that loads the `TypingSession`
                        let session_loader = LoadingScreen::load(move || {
                            Self::create_session(source, words_amount, max_word_length)
                                .map(|session| Message::Show(session.boxed()))
                        })
                        .boxed();

                        return Some(Message::Show(session_loader));
                    }
                    _ => (),
                }
            }
        }

        None
    }
}
