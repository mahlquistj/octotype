use std::{rc::Rc, time::Duration};

use crossterm::event::{self, Event};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, ToLine},
    widgets::Padding,
    Frame,
};

use crate::config::Config;
use crate::utils::{KeyEventHelper, ROUNDED_BLOCK};

mod error;
mod loadscreen;
mod menu;
mod session;

pub use loadscreen::LoadingScreen;
pub use menu::Menu;

pub trait Page {
    /// Renders the page. Called every cycle
    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config);

    /// Renders a line in the top left of the window.
    ///
    /// Called every cycle, before render.
    fn render_top(&mut self, _config: &Config) -> Option<Line> {
        None
    }

    /// Handles events for the page.
    ///
    /// Called every time an event appears, after render
    fn handle_events(&mut self, _event: &Event, _config: &Config) -> Option<Message> {
        None
    }

    /// Polls the page for any extra messages (e.g. loadingscreen finished).
    ///
    /// Called before handle_events
    fn poll(&mut self, _config: &Config) -> Option<Message> {
        None
    }

    /// Boxes the page
    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

/// An app message
///
/// This only has one variant for now, but keeping as an enum for future message-implementations
///
pub enum Message {
    Error(Box<dyn std::error::Error + Send>),
    Show(Box<dyn Page + Send>),
}

/// The app itself
pub struct App {
    page: Box<dyn Page>,
    config: Rc<Config>,
}

impl App {
    /// Creates a new `App`
    pub fn new(config: Config) -> Self {
        Self {
            page: Menu::new().boxed(),
            config: Rc::new(config),
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
            Message::Error(error) => self.page = error::Error::from(error).boxed(),
            Message::Show(page) => self.page = page,
        }

        false
    }
}
