use crossterm::event::{self, Event};
use ratatui::{style::Stylize, text::ToLine, widgets::Padding, DefaultTerminal, Frame};
use std::time::Duration;

use crate::utils::{KeyEventHelper, Message, Page, ROUNDED_BLOCK};

mod loadscreen;
mod menu;

pub use loadscreen::LoadingScreen;
pub use menu::Menu;
pub use menu::SourceError;

#[derive(Default)]
pub struct App {
    menu: Menu,
    page: Option<Box<dyn Page>>,
    // TODO: Is it possible to avoid using loadscreen (and maybe also menu) directly, and use it
    // as a normal page instead, when we need to consume the joinhandle?
    loading: Option<LoadingScreen>,
    // TODO:
    config: (),
}

impl App {
    pub fn new() -> Self {
        App::default()
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

        // TODO: REFACTOR
        if let Some(page) = &mut self.page {
            if let Some(top_msg) = page.render_top() {
                block = block.title_top(top_msg);
            }
            frame.render_widget(block, area);
            page.render(frame, content);
        } else if let Some(loader) = &mut self.loading {
            frame.render_widget(block, area);
            loader.render(frame, area);
        } else {
            frame.render_widget(block, area);
            self.menu.render(frame, content);
        }
    }

    fn handle_events(&mut self, event_opt: Option<Event>) -> std::io::Result<bool> {
        if let Some(msg) = self.loading.as_mut().and_then(|l| l.poll()) {
            return Ok(self.handle_message(msg));
        }

        if let Some(event) = event_opt {
            if let Some(page) = &mut self.page {
                if let Some(msg) = page.handle_events(&event) {
                    return Ok(self.handle_message(msg));
                }
            } else if self.loading.is_none() {
                if let Some(msg) = self.menu.handle_events(&event) {
                    return Ok(self.handle_message(msg));
                }
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
            Message::Show(page) => self.page = Some(page),
            Message::Await(loadscreen) => {
                self.loading = Some(loadscreen);
            }
            Message::ShowLoaded => {
                let loaded = self.loading.take().expect("Nothing was loading").join();
                return self.handle_message(loaded);
            }
            Message::ShowMenu => self.page = None,
        }

        false
    }
}
