use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
    DefaultTerminal, Frame,
};
use std::time::Duration;

use crate::session::{Stats, TypingSession};
use crate::{session::Library, utils::KeyEventHelper};

mod menu;

use menu::Menu;

pub struct App {
    menu: Menu,
    session: Option<TypingSession>,
    stats: Option<Stats>,
    config: (),
}

impl App {
    pub fn new() -> Self {
        Self {
            menu: Menu,
            session: None,
            stats: None,
            config: (),
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        self.session = Some(Library::get_words(10, None).await.expect("Error"));

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if self.handle_events()? {
                break;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [title, content] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

        frame.render_widget(Block::bordered().title("Typers"), title);

        if let Some(session) = &mut self.session {
            session.render(frame, content).expect("SESSION ERROR");
        } else if let Some(stat) = &mut self.stats {
            stat.render(frame, content).expect("STATS ERROR");
        }


    }

    fn handle_events(&mut self) -> std::io::Result<bool> {
        let event = if event::poll(Duration::ZERO)? {
            Some(event::read()?)
        } else {
            None
        };

        match (&mut self.session, event) {
            (_, Some(Event::Key(key))) if key.is_ctrl_press() => {
                return Self::handle_global_key_events(key);
            }
            (Some(session), key) => {
                if let Some(stats) = session.poll() {
                    println!("STATS");
                    self.session = None;
                    self.stats = Some(stats);
                    return Ok(false);
                }

                if let Some(Event::Key(key_event)) = key {
                    Self::handle_session_key_events(session, key_event);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_session_key_events(session: &mut TypingSession, key: KeyEvent) {
        if key.is_press() {
            match key.code {
                // Add character
                KeyCode::Char(character) => session.add(character),
                // Delete character
                KeyCode::Backspace => session.delete_input(),
                _ => {}
            }
        }
    }

    fn handle_global_key_events(key: KeyEvent) -> std::io::Result<bool> {
        if let KeyCode::Char('q') = key.code {
            return Ok(true);
        }

        Ok(false)
    }
}
