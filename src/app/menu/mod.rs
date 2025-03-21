use std::{fmt::Display, str::FromStr};

use super::{
    session::{Segment, TypingSession},
    LoadingScreen,
};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListState, Padding, Paragraph},
};
use strum::{IntoEnumIterator, VariantNames};

use crate::{
    config::Config,
    utils::{center, KeyEventHelper, Message, Page},
};

mod sources;
use sources::{Source, SourceError};

/// Page: Main menu
pub struct Menu {
    source_variants: Vec<String>,
    source: Source,
    words_amount: u32,
    max_word_length: u32,
    source_list_state: ListState,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            source_variants: Source::VARIANTS.iter().map(|s| s.to_string()).collect(),
            source: Source::default(),
            words_amount: 10,
            max_word_length: 0,
            source_list_state: ListState::default(),
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

        let words = words
            .chunks(5)
            .map(|words| {
                let string = words
                    .iter()
                    .cloned()
                    .map(|mut word| {
                        word.push(' ');
                        word
                    })
                    .collect::<String>();

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

        if let Some(selected) = self.source_list_state.selected() {
            let current_str: &'static str = self.source.clone().into();
            let selected_str = &self.source_variants[selected];
            if selected_str != current_str {
                self.source = Source::from_str(selected_str).expect("Unknown variant");
            }
        }

        // TODO: Refactor away the clone. Maybe create own VARIANTS const on Source
        let list = List::new(self.source_variants.clone())
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">");

        let [source_area, text_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(block.inner(center));

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

        frame.render_stateful_widget(list, source_area, &mut self.source_list_state);
        frame.render_widget(settings, text_area);
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

                    KeyCode::Tab => self.source_list_state.select_next(),

                    KeyCode::Enter => {
                        let source = self.source.clone();
                        let words_amount = self.words_amount;

                        let max_word_length =
                            (self.max_word_length > 0).then_some(self.max_word_length);

                        // Spawn a `LoadingScreen` that loads the `TypingSession`
                        let session_loader = LoadingScreen::load("Loading words...", move || {
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
