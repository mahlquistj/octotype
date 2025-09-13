use super::{Message, loadscreen::Loading, session::TypingSession};

use crossterm::event::{Event, KeyCode};
use derive_more::From;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, Padding, Paragraph},
};
use thiserror::Error;

use crate::{
    app::State,
    config::{
        Config, ModeConfig, SourceConfig,
        parameters::{Definition, Parameter},
    },
    page::session::{CreateModeError, FetchError, Mode},
    utils::center,
};

#[derive(Debug, Error, From)]
pub enum CreateSessionError {
    #[error("{0}")]
    Mode(Box<CreateModeError>),

    #[error("Failed to create session: {0}")]
    Fetch(FetchError),
}

/// Page: Main menu
#[derive(Debug)]
pub enum Menu {
    ModeSelect {
        mode_index: usize,
        modes: Vec<String>,
    },
    SourceSelect {
        selected_mode: String,
        source_index: usize,
        sources: Vec<String>,
    },
    ParameterConfig {
        mode: Box<ModeConfig>,
        source: Box<SourceConfig>,
        parameters: Vec<(String, Parameter)>,
        param_index: usize,
    },
}

impl Menu {
    /// Creates a new menu
    pub fn new(config: &Config) -> Self {
        Self::ModeSelect {
            mode_index: 0,
            modes: config.list_modes(),
        }
    }
}

// Rendering logic
impl Menu {
    pub fn render(&self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect, state: &State) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));
        let inner = block.inner(center);
        let config = &state.config;

        match &self {
            Self::ModeSelect { mode_index, modes } => {
                self.render_mode_select(frame, inner, config, modes, *mode_index);
            }
            Self::SourceSelect {
                selected_mode,
                source_index,
                sources,
                ..
            } => {
                self.render_source_select(
                    frame,
                    inner,
                    config,
                    selected_mode,
                    sources,
                    *source_index,
                );
            }
            Self::ParameterConfig {
                mode,
                source,
                parameters,
                param_index,
            } => {
                self.render_parameter_config(
                    frame,
                    inner,
                    config,
                    &mode.meta.name,
                    &source.meta.name,
                    parameters,
                    *param_index,
                );
            }
        }
    }

    fn render_mode_select(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
        modes: &[String],
        index: usize,
    ) {
        // TODO: Render selected mode description
        let items: Vec<Line> = modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                let style = if i == index {
                    Style::new()
                        .fg(config.settings.theme.text.highlight)
                        .reversed()
                } else {
                    Style::new()
                };
                Line::from(Span::styled(format!("{}. {}", i + 1, mode), style))
            })
            .collect();

        let list = List::new(items).block(Block::default().title("Select Mode"));

        frame.render_widget(list, area);
    }

    fn render_source_select(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
        mode_name: &str,
        sources: &[String],
        index: usize,
    ) {
        let items: Vec<Line> = sources
            .iter()
            .enumerate()
            .map(|(i, source)| {
                let style = if i == index {
                    Style::new()
                        .fg(config.settings.theme.text.highlight)
                        .reversed()
                } else {
                    Style::new()
                };
                Line::from(Span::styled(format!("{}. {}", i + 1, source), style))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(format!("Select Source for {}", mode_name)));

        frame.render_widget(list, area);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_parameter_config(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
        mode_name: &str,
        source_name: &str,
        params: &[(String, Parameter)],
        index: usize,
    ) {
        let mut lines = vec![
            Line::from(format!("Configuring {} -> {}", mode_name, source_name)),
            Line::from(""),
        ];

        for (i, (name, parameter)) in params.iter().filter(|(_, p)| p.is_mutable()).enumerate() {
            let style = if i == index {
                Style::new()
                    .fg(config.settings.theme.text.highlight)
                    .reversed()
            } else {
                Style::new()
            };
            lines.push(Line::from(Span::styled(
                format!("{}: {}", name, parameter.get_value()),
                style,
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("Press Enter to start typing session"));

        let paragraph = Paragraph::new(lines)
            .block(Block::default().title("Configure Parameters"))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    pub fn handle_events(
        &mut self,
        event: &crossterm::event::Event,
        state: &State,
    ) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            match self {
                Self::ModeSelect { mode_index, modes } => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            increment_index(mode_index, modes.len())
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            decrement_index(mode_index, modes.len())
                        }
                        KeyCode::Enter => {
                            // SAFETY: The index is always within range of the `modes` Vec
                            let mode_name = modes.get(*mode_index).unwrap();

                            if let Some(mode) = state.config.modes.get(mode_name) {
                                let sources = mode
                                    .sources
                                    .clone()
                                    .unwrap_or_else(|| state.config.list_sources());

                                *self = Self::SourceSelect {
                                    selected_mode: mode_name.clone(),
                                    sources,
                                    source_index: 0,
                                };
                            }
                        }
                        _ => (),
                    }
                }
                Self::SourceSelect {
                    selected_mode,
                    source_index,
                    sources,
                } => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        increment_index(source_index, sources.len())
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        decrement_index(source_index, sources.len())
                    }
                    KeyCode::Enter => {
                        let selected_source = sources[*source_index].clone();
                        let mut source = state.config.sources.get(&selected_source)?.clone();
                        let mut mode = state.config.modes.get(selected_mode)?.clone();
                        let mut source_overrides = mode.source_overrides.get_mut(&selected_source);

                        let mut parameters = Vec::new();

                        for (name, mut definition) in
                            source.parameters.drain().chain(mode.parameters.drain())
                        {
                            let mut mutable = true;
                            if let Some(ref mut overrides) = source_overrides
                                && let Some(override_param) = overrides.remove(&name)
                            {
                                mutable = false;
                                definition = Definition::FixedString(override_param);
                            }

                            let parameter = match definition.into_parameter(mutable) {
                                Ok(p) => p,
                                Err(error) => return Some(Message::Error(Box::new(error))),
                            };

                            parameters.push((name, parameter));
                        }

                        *self = Self::ParameterConfig {
                            mode: Box::new(mode),
                            source: Box::new(source),
                            parameters,
                            param_index: 0,
                        };
                    }
                    KeyCode::Backspace => {
                        *self = Self::ModeSelect {
                            mode_index: 0,
                            modes: state.config.list_modes(),
                        };
                    }
                    _ => {}
                },
                Self::ParameterConfig {
                    mode,
                    source,
                    parameters,
                    param_index,
                } => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            increment_index(param_index, parameters.len())
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            decrement_index(param_index, parameters.len())
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            parameters[*param_index].1.increment()
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            parameters[*param_index].1.decrement()
                        }
                        KeyCode::Enter => {
                            let mode = mode.clone();
                            let source = source.clone();
                            let parameters = parameters.clone().into_iter().collect();
                            let sources_dir = state
                                .config
                                .settings
                                .sources_dir
                                .clone()
                                .unwrap_or_default();
                            let session_loader = Loading::load("Loading words...", move || {
                                // TODO: Create a shared error type for creating sessions, so we
                                // can get rid of the unwrap
                                let mode =
                                    Mode::from_config(sources_dir, *mode, *source, parameters)
                                        .map_err(Box::new)?;
                                TypingSession::new(mode)
                                    .map(|session| Message::Show(session.into()))
                                    .map_err(CreateSessionError::from)
                            });

                            return Some(Message::Show(session_loader.into()));
                        }
                        KeyCode::Backspace => {
                            // Go back to source selection
                            *self = Self::SourceSelect {
                                selected_mode: mode.meta.name.clone(),
                                source_index: 0,
                                sources: state.config.list_sources(),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }
}

const fn increment_index(index: &mut usize, len: usize) {
    *index = if *index == 0 { len - 1 } else { *index - 1 }
}

const fn decrement_index(index: &mut usize, len: usize) {
    *index = (*index + 1) % len
}
