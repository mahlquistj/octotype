use super::{Message, loadscreen::Loading, session::Session};

use crossterm::event::{Event, KeyCode, KeyEvent};
use derive_more::From;
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List},
};
use thiserror::Error;

use crate::{
    config::{
        Config, ModeConfig, SourceConfig,
        parameters::{Definition, Parameter},
    },
    page::session::{CreateModeError, FetchError, Mode},
    utils::{center, centered_padding},
};

#[derive(Debug, Error, From)]
pub enum CreateSessionError {
    #[error("{0}")]
    Mode(Box<CreateModeError>),

    #[error("Failed to create session: {0}")]
    Fetch(FetchError),
}

/// Page: Main menu
#[derive(Debug, Clone, Copy)]
enum State {
    ModeSelect,
    SourceSelect,
    ParameterConfig,
}

#[derive(Debug)]
struct Context {
    modes: Vec<String>,
    sources: Vec<String>,
    selected_mode: Option<Box<ModeConfig>>,
    selected_source: Option<Box<SourceConfig>>,
    parameters: Vec<(String, Parameter)>,
    mode_index: usize,
    source_index: usize,
    param_index: usize,
}

impl Context {
    fn new(config: &Config) -> Self {
        Self {
            modes: config.list_modes(),
            sources: config.list_sources(),
            selected_mode: None,
            selected_source: None,
            parameters: vec![],
            mode_index: 0,
            source_index: 0,
            param_index: 0,
        }
    }
}

#[derive(Debug)]
pub struct Menu {
    state: State,
    context: Context,
}

impl Menu {
    /// Creates a new menu
    pub fn new(config: &Config) -> Self {
        Self {
            state: State::ModeSelect,
            context: Context::new(config),
        }
    }
}

// Rendering logic
impl Menu {
    pub fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        match &self.state {
            State::ModeSelect => {
                self.render_mode_select(frame, area, config);
            }
            State::SourceSelect => {
                self.render_source_select(frame, area, config);
            }
            State::ParameterConfig => {
                self.render_parameter_config(frame, area, config);
            }
        }
    }

    pub fn handle_events(&mut self, event: &Event, config: &Config) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            return match self.state {
                State::ModeSelect => self.handle_mode_select(key, config),
                State::SourceSelect => self.handle_source_select(key, config),
                State::ParameterConfig => self.handle_parameter_config(key, config),
            };
        }

        None
    }
}

// Render helpers
impl Menu {
    fn render_mode_select(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let index = self.context.mode_index;
        let items = self.context.modes.iter().enumerate().map(|(i, mode)| {
            let mut selector = "  ";
            let style = if i == index {
                selector = "> ";
                Style::new()
                    .fg(config.settings.theme.text.highlight)
                    .reversed()
            } else {
                Style::new()
            };
            Line::from(Span::styled(format!("{selector}{mode}"), style))
        });

        let list = List::new(items);
        let padding = centered_padding(area, Some(list.len() as u16 + 1), None);
        let area = Block::new().padding(padding).inner(area);

        frame.render_widget(list.block(Block::new().title("Select Mode")), area);
    }

    fn render_source_select(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let mode = self.context.selected_mode.as_ref().unwrap();
        let index = self.context.source_index;
        let items = self.context.sources.iter().enumerate().map(|(i, source)| {
            let mut selector = "  ";
            let style = if i == index {
                selector = "> ";
                Style::new()
                    .fg(config.settings.theme.text.highlight)
                    .reversed()
            } else {
                Style::new()
            };
            Line::from(Span::styled(format!("{selector}{source}"), style))
        });

        let list = List::new(items);
        let padding = centered_padding(area, Some(list.len() as u16 + 1), None);
        let area = Block::new().padding(padding).inner(area);

        frame.render_widget(
            list.block(Block::default().title(format!("Select Source for {}", mode.meta.name))),
            area,
        );
    }

    fn render_parameter_config(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let mode = self.context.selected_mode.as_ref().unwrap();
        let source = self.context.selected_source.as_ref().unwrap();
        let index = self.context.param_index;

        let items = self
            .context
            .parameters
            .iter()
            .filter(|(_, p)| p.is_mutable())
            .enumerate()
            .map(|(i, (name, parameter))| {
                let mut selector = "  ";
                let style = if i == index {
                    selector = "> ";
                    Style::new()
                        .fg(config.settings.theme.text.highlight)
                        .reversed()
                } else {
                    Style::new()
                };
                Line::from(Span::styled(
                    format!("{selector}{name}: {}", parameter.get_value()),
                    style,
                ))
            });

        let list = List::new(items);
        let padding = centered_padding(area, Some(list.len() as u16 + 1), None);
        let area = Block::new().padding(padding).inner(area);

        frame.render_widget(
            list.block(Block::default().padding(padding).title(format!(
                "Configuring Mode '{}' with Source '{}'",
                mode.meta.name, source.meta.name
            ))),
            area,
        );
    }
}

// Event handlers
impl Menu {
    fn handle_mode_select(&mut self, key: &KeyEvent, config: &Config) -> Option<Message> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                increment_index(&mut self.context.mode_index, self.context.modes.len())
            }
            KeyCode::Down | KeyCode::Char('j') => {
                decrement_index(&mut self.context.mode_index, self.context.modes.len())
            }
            KeyCode::Enter => {
                // SAFETY: The index is always within range of the `modes` Vec
                let mode_name = &self.context.modes[self.context.mode_index];
                if let Some(mode) = config.modes.get(mode_name) {
                    self.context.selected_mode = Some(Box::new(mode.clone()));
                    self.state = State::SourceSelect;
                }
            }
            _ => (),
        };

        None
    }

    fn handle_source_select(&mut self, key: &KeyEvent, config: &Config) -> Option<Message> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                increment_index(&mut self.context.source_index, self.context.sources.len())
            }
            KeyCode::Down | KeyCode::Char('j') => {
                decrement_index(&mut self.context.source_index, self.context.sources.len())
            }
            KeyCode::Enter => {
                let selected_source = &self.context.sources[self.context.source_index];
                let source = config.sources.get(selected_source).unwrap().clone();
                let mode = self.context.selected_mode.as_ref().unwrap();
                let source_overrides = mode.overrides.get(selected_source);

                let mut parameters = Vec::new();

                for (name, definition) in source.parameters.iter().chain(mode.parameters.iter()) {
                    let mut definition = definition.clone();
                    let mut mutable = true;
                    if let Some(overrides) = source_overrides
                        && let Some(override_param) = overrides.get(name)
                    {
                        mutable = false;
                        definition = Definition::FixedString(override_param.clone());
                    }

                    let parameter = match definition.into_parameter(mutable) {
                        Ok(p) => p,
                        Err(error) => return Some(Message::Error(Box::new(error))),
                    };

                    parameters.push((name.clone(), parameter));
                }

                self.context.selected_source = Some(Box::new(source));

                if parameters.is_empty() {
                    return self.create_session(config);
                }

                self.context.parameters = parameters;
                self.state = State::ParameterConfig;
            }
            KeyCode::Backspace => {
                self.context.selected_mode = None;
                self.state = State::ModeSelect;
            }
            _ => (),
        };

        None
    }

    fn handle_parameter_config(&mut self, key: &KeyEvent, config: &Config) -> Option<Message> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                increment_index(&mut self.context.param_index, self.context.parameters.len())
            }
            KeyCode::Down | KeyCode::Char('j') => {
                decrement_index(&mut self.context.param_index, self.context.parameters.len())
            }
            KeyCode::Right | KeyCode::Char('l') => self.context.parameters
                [self.context.param_index]
                .1
                .increment(),
            KeyCode::Left | KeyCode::Char('h') => self.context.parameters[self.context.param_index]
                .1
                .decrement(),
            KeyCode::Enter => {
                return self.create_session(config);
            }
            KeyCode::Backspace => {
                // Go back to source selection
                self.context.selected_source = None;
                self.state = State::SourceSelect;
            }
            _ => (),
        };

        None
    }

    fn create_session(&self, config: &Config) -> Option<Message> {
        let mode = *self.context.selected_mode.as_ref().unwrap().clone();
        let source = *self.context.selected_source.as_ref().unwrap().clone();
        let parameters = self.context.parameters.iter().cloned().collect();
        let session_loader = Loading::load(config, "Loading words...", move |config| {
            let mode = Mode::from_config(config, mode, source, parameters).map_err(Box::new)?;
            Session::new(config, mode)
                .map(|session| Message::Show(session.into()))
                .map_err(CreateSessionError::from)
        });

        Some(Message::Show(session_loader.into()))
    }
}

const fn increment_index(index: &mut usize, len: usize) {
    *index = if *index == 0 { len - 1 } else { *index - 1 }
}

const fn decrement_index(index: &mut usize, len: usize) {
    *index = (*index + 1) % len
}
