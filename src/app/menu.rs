use crossterm::event::{Event, KeyCode};
use ratatui::text::Line;

use crate::{
    session::Library,
    utils::{KeyEventHelper, Message, Page},
};

#[derive(Default)]
pub struct Menu;

impl Page for Menu {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let text = Line::from("Press S to start a new session");

        frame.render_widget(text, area);
    }

    fn handle_events(&mut self, event: &crossterm::event::Event) -> crate::utils::EventResult {
        if let Event::Key(key) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Char('s') => {
                        let session = std::thread::spawn(|| Library::get_words(10, None));
                        return Ok(Some(Message::Await(session)));
                    }
                    _ => todo!(),
                }
            }
        }

        Ok(None)
    }
}
