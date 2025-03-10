use crossterm::event::{self, Event, KeyCode};
use ratatui::{style::Stylize, text::ToLine, widgets::Padding, DefaultTerminal, Frame};
use std::time::Duration;

use crate::utils::{KeyEventHelper, Message, Page, ROUNDED_BLOCK};

mod loadscreen;
mod menu;

use loadscreen::LoadingScreen;
use menu::Menu;

pub struct App {
    page: Box<dyn Page>,
    // TODO: Is it possible to avoid using loadscreen directly, and us it
    // as a normal page instead, when we need to consume the joinhandle?
    loading: Option<LoadingScreen>,
    // TODO:
    config: (),
}

impl App {
    pub fn new() -> Self {
        Self {
            page: Menu.boxed(),
            loading: None,
            config: (),
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

        if let Some(loading_screen) = &mut self.loading {
            frame.render_widget(block, area);
            loading_screen.render(frame, content);
        } else {
            if let Some(top_msg) = self.page.render_top() {
                block = block.title_top(top_msg);
            }
            frame.render_widget(block, area);
            self.page.render(frame, content);
        }
    }

    fn handle_events(&mut self, event_opt: Option<Event>) -> std::io::Result<bool> {
        if let Some(msg) = self.loading.as_mut().and_then(|l| l.poll()) {
            return Ok(self.handle_message(msg));
        }

        if let Some(event) = event_opt {
            if let Some(msg) = self.page.handle_events(&event)? {
                return Ok(self.handle_message(msg));
            }

            match event {
                Event::Key(key) if key.is_ctrl_press() => {
                    if let KeyCode::Char('q') = key.code {
                        return Ok(true);
                    }
                }
                _ => (),
            }
        }

        Ok(false)
    }

    fn handle_message(&mut self, msg: Message) -> bool {
        match msg {
            Message::Quit => return true,
            Message::Show(page) => self.page = page,
            Message::Await(handle) => {
                self.loading = Some(LoadingScreen::load(handle));
            }
            Message::ShowLoaded => {
                let loaded = self.loading.take().expect("Nothing was loading").join();
                return self.handle_message(loaded);
            }
        }

        false
    }
}
