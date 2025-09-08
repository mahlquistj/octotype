use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::{Frame, style::Stylize, text::ToLine, widgets::Padding};

use crate::config::Config;
use crate::page;
use crate::session_factory::SessionFactory;
use crate::utils::{KeyEventHelper, ROUNDED_BLOCK};

/// An app message
///
/// This only has one variant for now, but keeping as an enum for future message-implementations
///
pub enum Message {
    Error(Box<dyn std::error::Error + Send>),
    Show(page::Page),
    Reset,
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

        if let Some(top_msg) = self.page.render_top(&self.state) {
            block = block.title_top(top_msg);
        }

        frame.render_widget(block, area);

        self.page.render(frame, content, &self.state);
    }

    /// Global event handler
    fn handle_events(&mut self, event_opt: Option<Event>) -> std::io::Result<bool> {
        if let Some(msg) = self.page.poll(&self.state) {
            return Ok(self.handle_message(msg));
        }

        if let Some(event) = event_opt {
            if let Some(msg) = self.page.handle_events(&event, &self.state) {
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
            Message::Reset => self.page = page::Menu::new(&self.state.session_factory).into(),
        }

        false
    }
}
