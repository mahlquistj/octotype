use std::str::FromStr;

use super::{
    Message,
    loadscreen::Loading,
    session::{Segment, TypingSession},
};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListState, Padding, Paragraph, Wrap},
};
use strum::VariantNames;

use crate::sources::{self, Source, SourceError};
use crate::{config::Config, utils::center};

/// Page: Main menu
pub struct Menu {
    source_variants: Vec<String>,
    source_list_state: ListState,
    source: Source,
    args: sources::Args,
    args_list_state: ListState,
}

impl Default for Menu {
    fn default() -> Self {
        let menu = Self {
            source_variants: Source::VARIANTS.iter().map(|s| s.to_string()).collect(),
            source_list_state: ListState::default().with_selected(Some(0)),
            source: Source::default(),
            args: Source::default().get_default_args(),
            args_list_state: ListState::default(),
        };

        menu
    }
}

impl Menu {
    /// Creates a new menu
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `TypingSession` with the given parameters
    fn create_session(source: Source, args: sources::Args) -> Result<TypingSession, SourceError> {
        let words = source.fetch(args)?;

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

        Ok(TypingSession::new(words)?)
    }
}

// Rendering logic
impl Menu {
    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        if let Some(selected_source) = self.source_list_state.selected() {
            let current_str: &'static str = self.source.clone().into();
            let selected_str = &self.source_variants[selected_source];
            if selected_str != current_str {
                self.source = Source::from_str(selected_str).expect("Unknown variant");
                self.args = self.source.get_default_args();

                if !self.args.is_empty() {
                    self.args_list_state.select(Some(0));
                }
            }
        }

        // TODO: Refactor away the clone. Maybe create own VARIANTS const on Source
        let list = List::new(self.source_variants.clone())
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">");

        let [source_area, text_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(block.inner(center));

        let lines = self
            .args
            .iter()
            .enumerate()
            .map(|(idx, (name, value))| {
                let mut value_text = value.render();
                let mut name_text = Span::raw(format!("{name}:"));

                if let Some(selected_arg) = self.args_list_state.selected() {
                    if selected_arg == idx {
                        name_text = name_text.bold().fg(config.theme.text.highlight)
                    }
                }

                let mut text = vec![name_text];
                text.append(&mut value_text);

                Line::from(text)
            })
            .collect::<Vec<Line>>();

        let settings = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_stateful_widget(list, source_area, &mut self.source_list_state);
        frame.render_widget(settings, text_area);
    }

    pub fn handle_events(
        &mut self,
        event: &crossterm::event::Event,
        _config: &Config,
    ) -> Option<Message> {
        if let Event::Key(key) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Tab => {
                        self.source_list_state.select_next();

                        if Some(self.source_variants.len()) == self.source_list_state.selected() {
                            self.source_list_state.select(Some(0));
                        }
                    }
                    KeyCode::Up => {
                        if !self.args.is_empty() {
                            if Some(0) == self.args_list_state.selected() {
                                self.args_list_state.select(Some(self.args.len() - 1));
                            } else {
                                self.args_list_state.select_previous();
                            }
                        }
                    }
                    KeyCode::Down => {
                        if !self.args.is_empty() {
                            self.args_list_state.select_next();

                            if Some(self.args.len()) == self.args_list_state.selected() {
                                self.args_list_state.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        let source = self.source.clone();

                        let args = std::mem::take(&mut self.args);
                        // Spawn a `LoadingScreen` that loads the `TypingSession`
                        let session_loader = Loading::load("Loading words...", move || {
                            Self::create_session(source, args)
                                .map(|session| Message::Show(session.into()))
                        });

                        return Some(Message::Show(session_loader.into()));
                    }
                    _ => (),
                }

                if let Some(selected_arg) = self.args_list_state.selected() {
                    if let Ok(setting_event) = key.try_into() {
                        let arg = &mut self.args[selected_arg].1;
                        arg.update(&setting_event);
                    }
                } else if !self.args.is_empty() {
                    self.args_list_state.select(Some(0));
                }
            }
        }

        None
    }
}
