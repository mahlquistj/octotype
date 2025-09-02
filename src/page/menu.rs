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

use crate::{
    config::Config,
    modes::ModeConfig,
    sources::{ParameterValues, SourceError},
    utils::center,
};

#[derive(Debug)]
pub enum MenuState {
    ModeSelect {
        modes: Vec<String>,
        selected_index: usize,
    },
    SourceSelect {
        mode_name: String,
        mode_params: ParameterValues,
        available_sources: Vec<String>,
        selected_index: usize,
    },
    ParameterConfig {
        mode_name: String,
        mode_params: ParameterValues,
        source_name: String,
        source_params: ParameterValues,
        editing_param_index: Option<usize>,
        param_names: Vec<String>,
    },
}

/// Page: Main menu
pub struct Menu {
    state: MenuState,
    mode_configs: Vec<ModeConfig>,
    available_sources: Vec<String>,
}

impl Menu {
    /// Creates a new menu
    pub fn new(mode_configs: Vec<ModeConfig>, available_sources: Vec<String>) -> Self {
        let mode_names: Vec<String> = mode_configs.iter().map(|m| m.name.clone()).collect();

        Self {
            state: MenuState::ModeSelect {
                modes: mode_names,
                selected_index: 0,
            },
            mode_configs,
            available_sources,
        }
    }

    /// Create a `TypingSession` with the given parameters
    fn create_session(words: Vec<String>) -> Result<TypingSession, SourceError> {
        let segments = words
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

        Ok(TypingSession::new(segments)?)
    }

    fn get_mode_config(&self, name: &str) -> Option<&ModeConfig> {
        self.mode_configs.iter().find(|m| m.name == name)
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
        let inner = block.inner(center);

        match &self.state {
            MenuState::ModeSelect {
                modes,
                selected_index,
            } => {
                self.render_mode_select(frame, inner, config, modes, *selected_index);
            }
            MenuState::SourceSelect {
                mode_name,
                available_sources,
                selected_index,
                ..
            } => {
                self.render_source_select(
                    frame,
                    inner,
                    config,
                    mode_name,
                    available_sources,
                    *selected_index,
                );
            }
            MenuState::ParameterConfig {
                mode_name,
                source_name,
                source_params,
                editing_param_index,
                param_names,
                ..
            } => {
                self.render_parameter_config(
                    frame,
                    inner,
                    config,
                    mode_name,
                    source_name,
                    source_params,
                    editing_param_index,
                    param_names,
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
        selected_index: usize,
    ) {
        let items: Vec<Line> = modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                let style = if i == selected_index {
                    Style::new().fg(config.theme.text.highlight).reversed()
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
        selected_index: usize,
    ) {
        let items: Vec<Line> = sources
            .iter()
            .enumerate()
            .map(|(i, source)| {
                let style = if i == selected_index {
                    Style::new().fg(config.theme.text.highlight).reversed()
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
        params: &ParameterValues,
        editing_index: &Option<usize>,
        param_names: &[String],
    ) {
        let mut lines = vec![
            Line::from(format!("Configuring {} -> {}", mode_name, source_name)),
            Line::from(""),
        ];

        for (i, param_name) in param_names.iter().enumerate() {
            let value = params
                .get_as_string(param_name)
                .unwrap_or_else(|| "<not set>".to_string());
            let style = if Some(i) == *editing_index {
                Style::new().fg(config.theme.text.highlight).reversed()
            } else {
                Style::new()
            };
            lines.push(Line::from(Span::styled(
                format!("{}: {}", param_name, value),
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
        _config: &Config,
    ) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            match &mut self.state {
                MenuState::ModeSelect {
                    modes,
                    selected_index,
                } => {
                    match key.code {
                        KeyCode::Up => {
                            *selected_index = if *selected_index == 0 {
                                modes.len() - 1
                            } else {
                                *selected_index - 1
                            };
                        }
                        KeyCode::Down => {
                            *selected_index = (*selected_index + 1) % modes.len();
                        }
                        KeyCode::Enter => {
                            let mode_name = modes[*selected_index].clone();
                            if let Some(mode_config) = self.get_mode_config(&mode_name) {
                                let mode_params = self.create_default_mode_params(mode_config);
                                let allowed_sources = mode_config
                                    .allowed_sources
                                    .clone()
                                    .unwrap_or_else(|| self.available_sources.clone());

                                self.state = MenuState::SourceSelect {
                                    mode_name,
                                    mode_params,
                                    available_sources: allowed_sources,
                                    selected_index: 0,
                                };
                            }
                        }
                        KeyCode::Esc => {
                            // Exit application or go back
                        }
                        _ => {}
                    }
                }
                MenuState::SourceSelect {
                    mode_name,
                    mode_params,
                    available_sources,
                    selected_index,
                } => {
                    match key.code {
                        KeyCode::Up => {
                            *selected_index = if *selected_index == 0 {
                                available_sources.len() - 1
                            } else {
                                *selected_index - 1
                            };
                        }
                        KeyCode::Down => {
                            *selected_index = (*selected_index + 1) % available_sources.len();
                        }
                        KeyCode::Enter => {
                            let source_name = available_sources[*selected_index].clone();
                            let source_params = ParameterValues::new(); // TODO: Create defaults
                            let param_names = vec![]; // TODO: Get actual parameter names

                            self.state = MenuState::ParameterConfig {
                                mode_name: mode_name.clone(),
                                mode_params: mode_params.clone(),
                                source_name,
                                source_params,
                                editing_param_index: None,
                                param_names,
                            };
                        }
                        KeyCode::Esc => {
                            let modes: Vec<String> =
                                self.mode_configs.iter().map(|m| m.name.clone()).collect();
                            self.state = MenuState::ModeSelect {
                                modes,
                                selected_index: 0,
                            };
                        }
                        _ => {}
                    }
                }
                MenuState::ParameterConfig { .. } => {
                    match key.code {
                        KeyCode::Enter => {
                            // Create session with current parameters
                            let words = vec![
                                "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
                            ]
                            .into_iter()
                            .map(String::from)
                            .collect();

                            let session_loader = Loading::load("Loading words...", move || {
                                Self::create_session(words)
                                    .map(|session| Message::Show(session.into()))
                            });

                            return Some(Message::Show(session_loader.into()));
                        }
                        KeyCode::Esc => {
                            // Go back to source selection
                            // TODO: Implement proper state transition
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    fn create_default_mode_params(&self, mode_config: &ModeConfig) -> ParameterValues {
        let mut params = ParameterValues::new();

        for (key, param_def) in &mode_config.parameters {
            match param_def {
                crate::sources::ParameterDefinition::Range { min, default, .. } => {
                    let value = default.unwrap_or(*min);
                    params.set_integer(key.clone(), value);
                }
                crate::sources::ParameterDefinition::Selection { default, .. } => {
                    params.set_string(key.clone(), default.clone());
                }
                crate::sources::ParameterDefinition::Toggle(default_value) => {
                    params.set_boolean(key.clone(), *default_value);
                }
            }
        }

        params
    }
}
