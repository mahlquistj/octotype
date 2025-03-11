use std::fmt::Display;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::Constraint,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    session::{Segment, TypingSession},
    utils::{center, KeyEventHelper, Message, Page},
};

use super::LoadingScreen;

#[derive(Debug)]
pub enum SourceError {
    CommonWords(std::io::Error),
    Request(minreq::Error),
}

impl From<std::io::Error> for SourceError {
    fn from(value: std::io::Error) -> Self {
        Self::CommonWords(value)
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
            Self::Request(e) => e.to_string(),
            Self::CommonWords(e) => e.to_string(),
        };

        write!(f, "{error}")
    }
}

impl std::error::Error for SourceError {}

#[derive(Clone)]
pub enum Source {
    CommonWords {
        // The language selected
        lang: String,
    },

    RandomApi {
        // The language selected (None = English)
        lang: Option<String>,
    },
}

impl Default for Source {
    fn default() -> Self {
        Self::RandomApi { lang: None }
    }
}

impl Source {
    fn fetch(&self, amount: usize, max_length: Option<usize>) -> Result<Vec<String>, SourceError> {
        let words = match self {
            Self::CommonWords { .. } => todo!("Implement commonwords"),
            Self::RandomApi { lang } => {
                let mut req = minreq::get("https://random-word-api.herokuapp.com/word")
                    .with_param("number", amount.to_string());
                // Add a header in case the api wants to block the app.
                // .with_header(
                //
                //     "typers_request",
                //     "CONTACT: https://github.com/madser123/typers",
                // );

                if let Some(language) = lang {
                    req = req.with_param("lang", language);
                }

                if let Some(ml) = max_length {
                    req = req.with_param("length", ml.to_string());
                }

                req.send()?.json::<Vec<String>>()?
            }
        };

        Ok(words)
    }
}

pub struct Menu {
    source: Source,
    words_amount: usize,
    max_word_length: usize,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            source: Source::RandomApi { lang: None },
            words_amount: 10,
            max_word_length: 0,
        }
    }
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_session(
        source: Source,
        words_amount: usize,
        max_word_length: Option<usize>,
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
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let text = vec![
            Line::from(vec![
                Span::raw(format!("Words     : {}", self.words_amount)),
                Span::styled("↕", Style::new().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw(format!("Max length: {}", self.max_word_length)),
                Span::styled("↔", Style::new().add_modifier(Modifier::BOLD)),
            ]),
        ];

        let settings = Paragraph::new(text);

        frame.render_widget(settings, center);
    }

    fn handle_events(&mut self, event: &crossterm::event::Event) -> Option<Message> {
        if let Event::Key(key) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Up => self.words_amount += 1,
                    KeyCode::Down => self.words_amount -= 1,

                    KeyCode::Right => self.max_word_length += 1,
                    KeyCode::Left => self.max_word_length -= 1,

                    KeyCode::Enter => {
                        let source = self.source.clone();
                        let words_amount = self.words_amount;
                        let max_word_length =
                            (self.max_word_length > 0).then_some(self.max_word_length);
                        let session_loader = LoadingScreen::load(move || {
                            Self::create_session(source, words_amount, max_word_length)
                                .map(|session| Message::Show(session.boxed()))
                        });
                        return Some(Message::Await(session_loader));
                    }
                    _ => (),
                }
            }
        }

        self.words_amount = self.words_amount.clamp(0, 50);
        self.max_word_length = self.max_word_length.clamp(0, 100);

        None
    }
}
