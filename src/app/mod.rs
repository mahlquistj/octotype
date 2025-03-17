use crossterm::event::{self, Event};
use ratatui::{style::Stylize, text::ToLine, widgets::Padding, DefaultTerminal, Frame};
use std::rc::Rc;
use std::time::Duration;

use crate::config::Config;
use crate::utils::{KeyEventHelper, Message, Page, ROUNDED_BLOCK};

mod loadscreen;
mod menu;

pub use loadscreen::LoadingScreen;
pub use menu::Menu;

pub struct App {
    page: Box<dyn Page>,
    config: Rc<Config>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            page: Menu::new().boxed(),
            config: Rc::new(config),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            let event = event::poll(Duration::ZERO)?.then(event::read).transpose()?;
            terminal.draw(|frame| self.draw(frame))?;
            if self.handle_events(event)? {
                break;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let mut block = ROUNDED_BLOCK
            .padding(Padding::new(1, 1, 0, 0))
            .title_top(
                "TYPERS - A lightweight TUI typing-test"
                    .to_line()
                    .bold()
                    .centered(),
            )
            .title_top("<CTRL-Q> to exit".to_line().right_aligned());

        let area = frame.area();
        let content = block.inner(area);

        if let Some(top_msg) = self.page.render_top(&self.config) {
            block = block.title_top(top_msg);
        }
        frame.render_widget(block, area);
        self.page.render(frame, content, &self.config);
    }

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

    fn handle_message(&mut self, msg: Message) -> bool {
        match msg {
            Message::Quit => return true,
            Message::Show(page) => self.page = page,
        }

        false
    }
}
