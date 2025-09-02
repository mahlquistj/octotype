use std::{rc::Rc, time::Duration};

use crossterm::event::{self, Event};
use ratatui::{Frame, style::Stylize, text::ToLine, widgets::Padding};

use crate::config::Config;
use crate::modes::ConditionValues;
use crate::page;
use crate::session_factory::SessionFactory;
use crate::sources::ParameterValues;
use crate::utils::{KeyEventHelper, ROUNDED_BLOCK};

/// An app message
///
/// This only has one variant for now, but keeping as an enum for future message-implementations
///
pub enum Message {
    Error(Box<dyn std::error::Error + Send>),
    Show(page::Page),
    CreateSession {
        mode_name: String,
        parameter_values: Option<ParameterValues>,
        condition_values: Option<ConditionValues>,
    },
}

/// The app itself
pub struct App {
    page: page::Page,
    config: Rc<Config>,
    session_factory: SessionFactory,
}

impl App {
    /// Creates a new `App`
    pub fn new(config: Config, session_factory: SessionFactory) -> Self {
        // Get mode configs and available sources from session factory
        let mode_configs = session_factory
            .get_mode_manager()
            .list_modes()
            .into_iter()
            .filter_map(|name| session_factory.get_mode_manager().get_mode(name).cloned())
            .collect();
        let available_sources = session_factory
            .get_source_manager()
            .list_sources()
            .into_iter()
            .map(String::from)
            .collect();

        Self {
            page: page::Menu::new(mode_configs, available_sources).into(),
            config: Rc::new(config),
            session_factory,
        }
    }

    /// Runs the app
    pub fn run(&mut self) -> std::io::Result<()> {
        let mut terminal = ratatui::init();

        loop {
            let event = event::poll(Duration::ZERO)?.then(event::read).transpose()?;
            terminal.draw(|frame| self.draw(frame))?;

            if self.handle_events(event)? {
                break; // Quit
            }
        }

        ratatui::restore();

        Ok(())
    }

    /// Draws the next frame
    fn draw(&mut self, frame: &mut Frame) {
        let mut block = ROUNDED_BLOCK
            .padding(Padding::new(1, 1, 0, 0))
            .title_top("OCTOTYPE".to_line().bold().centered())
            .title_top("<CTRL-Q> to exit".to_line().right_aligned());

        let area = frame.area();
        let content = block.inner(area);

        if let Some(top_msg) = self.page.render_top(&self.config) {
            block = block.title_top(top_msg);
        }

        frame.render_widget(block, area);

        self.page.render(frame, content, &self.config);
    }

    /// Global event handler
    fn handle_events(&mut self, event_opt: Option<Event>) -> std::io::Result<bool> {
        if let Some(msg) = self.page.poll(&self.config) {
            return Ok(self.handle_message(msg));
        }

        if let Some(event) = event_opt {
            if let Some(msg) = self.page.handle_events(&event, &self.config) {
                return Ok(self.handle_message(msg));
            }

            if let Event::Key(key) = event {
                return Ok(key.is_ctrl_press_char('q'));
            }
        }

        Ok(false)
    }

    /// Global message handler
    fn handle_message(&mut self, msg: Message) -> bool {
        match msg {
            Message::Error(error) => self.page = page::Error::from(error).into(),
            Message::Show(page) => self.page = page,
            Message::CreateSession {
                mode_name,
                parameter_values,
                condition_values,
            } => {
                let session = self.session_factory
                    .create_session(&mode_name, parameter_values, condition_values)
            }
        }

        false
    }
}
