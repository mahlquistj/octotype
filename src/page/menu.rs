use super::{
    Message,
    loadscreen::Loading,
    session::{TypingSession},
};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, Padding, Paragraph},
};

use crate::{
    app::State,
    config::{Config, ModeConfig, SourceConfig, parameters::Parameter},
    page::session::Mode,
    utils::center,
};

pub type ParameterValues = Vec<(String, Parameter)>;

#[derive(Debug)]
pub enum MenuState {
    ModeSelect {
        mode_index: usize,
    },
    SourceSelect {
        selected_mode: String,
        source_index: usize,
    },
    ParameterConfig {
        selected_mode: String,
        selected_source: String,
        parameters: ParameterValues,
        param_index: usize,
    },
}

/// Page: Main menu
pub struct Menu {
    state: MenuState,
    available_modes: Vec<String>,
    available_sources: Vec<String>,
}

impl Menu {
    /// Creates a new menu
    pub fn new(config: &Config) -> Self {
        Self {
            state: MenuState::ModeSelect { mode_index: 0 },
            available_modes: config.list_modes(),
            available_sources: config.list_sources(),
        }
    }
}

// Rendering logic
impl Menu {
    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &State,
    ) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));
        let inner = block.inner(center);
        let config = &state.config;

        match &self.state {
            MenuState::ModeSelect { mode_index} => {
                self.render_mode_select(
                    frame,
                    inner,
                    config,
                    &self.available_modes,
                    *mode_index,
                );
            }
            MenuState::SourceSelect {
                mode,
                available_sources,
                selected_index,
                ..
            } => {
                self.render_source_select(
                    frame,
                    inner,
                    config,
                    &mode.name,
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

        // TODO: Render selected mode description
        let items: Vec<Line> = modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                let style = if i == selected_index {
                    Style::new().fg(config.settings.theme.text.highlight).reversed()
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
        state: &State,
    ) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            match &mut self.state {
                MenuState::ModeSelect { selected_index } => {
                    let modes_len = self.available_modes.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            *selected_index = if *selected_index == 0 {
                                modes_len - 1
                            } else {
                                *selected_index - 1
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            *selected_index = (*selected_index + 1) % modes_len;
                        }
                        KeyCode::Enter => {
                            // TODO: Handle/return errors for when mode doesn't exist
                            let mode_name = self.available_modes.get(*selected_index)?;

                            if let Some(mode) = state
                                .session_factory
                                .mode_manager()
                                .get_mode(mode_name)
                                .cloned()
                            {
                                let allowed_sources =
                                    mode.allowed_sources.clone().unwrap_or_else(|| {
                                        state
                                            .session_factory
                                            .list_sources()
                                            .into_iter()
                                            .map(str::to_string)
                                            .collect()
                                    });

                                self.state = MenuState::SourceSelect {
                                    mode,
                                    available_sources: allowed_sources,
                                    selected_index: 0,
                                };
                            }
                        }
                        _ => {}
                    }
                }
                MenuState::SourceSelect {
                    mode,
                    available_sources,
                    selected_index,
                } => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            *selected_index = if *selected_index == 0 {
                                available_sources.len() - 1
                            } else {
                                *selected_index - 1
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            *selected_index = (*selected_index + 1) % available_sources.len();
                        }
                        KeyCode::Enter => {
                            let source_name = available_sources[*selected_index].clone();
                            let source_params = ParameterValues::new(); // TODO: Create defaults
                            let param_names = vec![]; // TODO: Get actual parameter names

                            self.state = MenuState::ParameterConfig {
                                mode_name: mode.name.clone(),
                                mode_params: create_default_mode_params(mode),
                                source_name,
                                source_params,
                                editing_param_index: None,
                                param_names,
                            };
                        }
                        KeyCode::Esc => {
                            self.state = MenuState::ModeSelect { selected_index: 0 };
                        }
                        _ => {}
                    }
                }
                MenuState::ParameterConfig {
                    mode_name,
                    mode_params,
                    source_name,
                    source_params,
                    ..
                } => {
                    match key.code {
                        KeyCode::Enter => {
                            let session_loader = Loading::load("Loading words...", move || {
                                create_session(mode_name, mode_params)
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
}

/// Create a `TypingSession` with the given parameters
fn create_session(name: ) -> Result<TypingSession, FetchError> {


    Ok(TypingSession::new(Mode {
        name: ,
        conditions: ,
        source: ,
    })?)
}

fn create_default_mode_params(mode_config: &ModeConfig) -> ParameterValues {
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
