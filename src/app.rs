use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, style::Stylize, text::ToLine, widgets::Padding};

use crate::config::Config;
use crate::page;
use crate::session_factory::SessionFactory;
use crate::utils::ROUNDED_BLOCK;

/// An app message
pub enum Message {
    /// An error occurred
    Error(Box<dyn std::error::Error + Send>),
    /// Show a specific page
    Show(page::Page),
    /// Reset to the main menu
    Reset,
    /// Quit the application
    Quit,
}

pub struct State {
    pub config: Config,
    pub session_factory: SessionFactory,
}

/// The app itself
pub struct App {
    page: page::Page,
    state: State,
}

impl App {
    /// Creates a new `App`
    pub fn new(config: Config, session_factory: SessionFactory) -> Self {
        Self {
            page: page::Menu::new(&session_factory).into(),
            state: State {
                config,
                session_factory,
            },
        }
    }

    /// Runs the app
    pub fn run(&mut self) -> std::io::Result<()> {
        let mut terminal = ratatui::init();

        loop {
            let event = event::poll(Duration::ZERO)?.then(event::read).transpose()?;
            terminal.draw(|frame| self.draw(frame))?;

            if let Some(message) = self.handle_events(event)
                && self.handle_message(message)
            {
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

        if let Some(top_msg) = self.page.render_top(&self.state) {
            block = block.title_top(top_msg);
        }

        frame.render_widget(block, area);

        self.page.render(frame, content, &self.state);
    }

    /// Global event handler
    fn handle_events(&mut self, event_opt: Option<Event>) -> Option<Message> {
        // Check for any events (keys, mouse, etc.)
        if let Some(event) = event_opt {
            // Global event handling
            match event {
                Event::Key(key) => return self.handle_key_event(key),
                _ => (), // Reserved for future event handling
            }

            // Page event handling
            if let Some(msg) = self.page.handle_events(&event, &self.state) {
                return Some(msg);
            }
        }

        // Poll the current page
        if let Some(msg) = self.page.poll(&self.state) {
            return Some(msg);
        }

        None
    }

    /// Global
    fn handle_key_event(&self, key: KeyEvent) -> Option<Message> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Message::Quit),
            (KeyCode::Esc, KeyModifiers::NONE) => Some(Message::Reset),
            _ => None,
        }
    }

    /// Global message handler
    ///
    /// Returns `true` if the application should quit
    fn handle_message(&mut self, msg: Message) -> bool {
        match msg {
            Message::Error(error) => self.page = page::Error::from(error).into(),
            Message::Show(page) => self.page = page,
            Message::Reset => self.page = page::Menu::new(&self.state.session_factory).into(),
            Message::Quit => return true,
        }

        false
    }
}
