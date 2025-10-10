use super::{History, Message, loadscreen::Loading, session::Session};

use crossterm::event::{Event, KeyCode, KeyEvent};
use derive_more::From;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, block::Title},
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
pub enum ContextError {
    #[error("No sources where found")]
    NoSources,

    #[error("No modes where found - You might not have any offline sources available")]
    NoModes,
}

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
    MainMenu,
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
    main_index: usize,
    mode_index: usize,
    source_index: usize,
    param_index: usize,
}

impl Context {
    fn new(config: &Config) -> Result<Self, ContextError> {
        let modes = config.list_modes();
        if modes.is_empty() {
            return Err(ContextError::NoModes);
        }

        let sources = config.list_sources();
        if sources.is_empty() {
            return Err(ContextError::NoSources);
        }

        Ok(Self {
            modes,
            sources,
            selected_mode: None,
            selected_source: None,
            parameters: vec![],
            main_index: 0,
            mode_index: 0,
            source_index: 0,
            param_index: 0,
        })
    }
}

#[derive(Debug)]
pub struct Menu {
    state: State,
    context: Context,
}

impl Menu {
    /// Creates a new menu
    pub fn new(config: &Config) -> Result<Self, ContextError> {
        Ok(Self {
            state: State::MainMenu,
            context: Context::new(config)?,
        })
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
            State::MainMenu => {
                self.render_main_menu(frame, area, config);
            }
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
                State::MainMenu => self.handle_main_menu(key, config),
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
    fn render_main_menu(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let main_menu_items = vec!["Start Typing Session", "View Statistics History"];
        let index = self.context.main_index;
        let items = main_menu_items.iter().map(|item| item.to_string());
        render_list(config, frame, items, "Main Menu", area, index);
    }
    fn render_mode_select(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let index = self.context.mode_index;
        let items = self.context.modes.iter().map(|mode| mode.to_string());
        render_list(config, frame, items, "Select mode", area, index);
    }

    fn render_source_select(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let mode = self.context.selected_mode.as_ref().unwrap();
        let index = self.context.source_index;
        let items = self.context.sources.iter().map(|source| source.to_string());
        let title = Line::from(vec![
            Span::raw("Select Source for Mode "),
            Span::raw(&mode.meta.name).bold(),
        ]);

        render_list(config, frame, items, title, area, index);
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
            .map(|(name, parameter)| format!("{name}: {}", parameter.get_value()));

        let title = Line::from(vec![
            Span::raw("Configuring Mode "),
            Span::raw(&mode.meta.name).bold(),
            Span::raw(" with Source "),
            Span::raw(&source.meta.name).bold(),
        ]);

        render_list(config, frame, items, title, area, index)
    }
}

// Event handlers
impl Menu {
    fn handle_main_menu(&mut self, key: &KeyEvent, config: &Config) -> Option<Message> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                increment_index(&mut self.context.main_index, 2) // 2 items in main menu
            }
            KeyCode::Down | KeyCode::Char('j') => {
                decrement_index(&mut self.context.main_index, 2) // 2 items in main menu
            }
            KeyCode::Enter => {
                match self.context.main_index {
                    0 => {
                        // Start Typing Session
                        self.state = State::ModeSelect;
                    }
                    1 => {
                        // View Statistics History
                        return match History::new(config) {
                            Ok(history) => Some(Message::Show(history.into())),
                            Err(error) => Some(Message::Error(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                error,
                            )))),
                        };
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        None
    }
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
            KeyCode::Backspace => {
                self.state = State::MainMenu;
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

fn render_list<'a>(
    config: &Config,
    frame: &mut ratatui::Frame,
    items: impl Iterator<Item = String>,
    title: impl Into<Title<'a>>,
    area: Rect,
    index: usize,
) {
    let items = items.enumerate().map(|(i, item)| {
        let mut selector = "  ";
        let style = if i == index {
            selector = "> ";
            Style::new()
                .fg(config.settings.theme.text.highlight)
                .reversed()
        } else {
            Style::new()
        };
        Line::from(vec![Span::raw(selector), Span::styled(item, style)])
    });
    let list = List::new(items);
    let padding = centered_padding(
        area,
        // + 1 to account for title
        Some(list.len() as u16 + 1),
        None,
    );
    let area = Block::new().padding(padding).inner(area);
    frame.render_widget(list.block(Block::default().title(title)), area);
}

const fn increment_index(index: &mut usize, len: usize) {
    *index = if *index == 0 { len - 1 } else { *index - 1 }
}

const fn decrement_index(index: &mut usize, len: usize) {
    *index = (*index + 1) % len
}
