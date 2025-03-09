use crossterm::event::{self, Event, KeyCode};
use ratatui::{style::Stylize, text::ToLine, widgets::Padding, DefaultTerminal, Frame};
use std::{fs::write, thread::JoinHandle, time::Duration};

use crate::utils::{KeyEventHelper, Message, Page, ROUNDED_BLOCK};

mod loadscreen;
mod menu;

use loadscreen::LoadingScreen;
use menu::Menu;

pub struct App {
    page: Box<dyn Page>,
    loading: Option<LoadingScreen>, // TODO: config: (),
}

impl App {
    pub fn new() -> Self {
        Self {
            page: Menu.boxed(),
            loading: None,
            // TODO: config: (),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            let event = event::poll(Duration::ZERO)?.then_some(event::read()?);
            terminal.draw(|frame| self.draw(frame))?;
            if self.handle_events(event)? {
                break;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let block = ROUNDED_BLOCK
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
        frame.render_widget(block, area);

        if let Some(loading_screen) = &mut self.loading {
            loading_screen.render(frame, content);
        } else {
            self.page.render(frame, content);
        }
    }

    fn handle_events(&mut self, event_opt: Option<Event>) -> std::io::Result<bool> {
        if let Some(event) = event_opt {
            if let Some(loading_screen) = &mut self.loading {
                if let Some(msg) = loading_screen.handle_events(&event)? {
                    return Ok(self.handle_message(msg));
                }
            }

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
