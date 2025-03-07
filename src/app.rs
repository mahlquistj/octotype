use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
    DefaultTerminal, Frame,
};
use std::time::Duration;

use crate::session::TypingSession;
use crate::{library::Library, utils::KeyEventHelper};

pub struct App {
    session: Option<TypingSession>,
    config: (),
}

impl App {
    pub fn new() -> Self {
        Self {
            session: None,
            config: (),
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<String> {
        self.session = Some(Library::get_words(10, None).await.expect("Error"));

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if self.handle_events()? {
                break;
            }
        }

        let wpm = self.session.as_ref().unwrap().calculate_wpm();

        Ok(format!("{wpm} Wpm"))
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [title, content] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

        frame.render_widget(Block::bordered().title("Typers"), title);

        if let Some(session) = &mut self.session {
            session.render(frame, content).expect("SESSION ERROR");
        }
    }

    fn handle_events(&mut self) -> std::io::Result<bool> {
        let event = if event::poll(Duration::ZERO)? {
            Some(event::read()?)
        } else {
            None
        };

        match (&mut self.session, event) {
            (Some(session), _) if session.is_done() => {
                return Ok(true);
            }
            (_, Some(Event::Key(key))) if key.is_ctrl_press() => {
                return Self::handle_global_key_events(key);
            }
            (Some(session), Some(Event::Key(key))) if key.is_press() => {
                Self::handle_session_key_events(session, key);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_session_key_events(session: &mut TypingSession, key: KeyEvent) {
        match key.code {
            // Add character
            KeyCode::Char(character) => session.add(&[character]),
            // Delete character
            KeyCode::Backspace => session.delete_input(),
            _ => {}
        }
    }

    fn handle_global_key_events(key: KeyEvent) -> std::io::Result<bool> {
        if let KeyCode::Char('q') = key.code {
            return Ok(true);
        }

        Ok(false)
    }
}
