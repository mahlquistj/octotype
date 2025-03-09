use std::thread::JoinHandle;

use ratatui::{layout::Constraint, style::Style};
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse, BRAILLE_SIX};

use crate::utils::{center, EventResult, Message, Page};

pub struct LoadingScreen {
    handle: JoinHandle<Result<Message, minreq::Error>>,
    state: ThrobberState,
}

impl LoadingScreen {
    pub fn load(handle: JoinHandle<Result<Message, minreq::Error>>) -> Self {
        Self {
            handle,
            state: ThrobberState::default(),
        }
    }

    pub fn join(self) -> Message {
        return match self.handle.join() {
            Ok(Ok(msg)) => msg,
            Ok(Err(error)) => todo!("Show error screen: {error}"),
            Err(handle_error) => todo!("Show error scrren: {handle_error:?}"),
        };
    }
}

impl Page for LoadingScreen {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        self.state.calc_next();

        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let throbber = Throbber::default()
            .label("Loading words...")
            .throbber_style(
                Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(BRAILLE_SIX)
            .use_type(WhichUse::Full);

        frame.render_widget(throbber, center);
    }

    fn handle_events(&mut self, _event: &crossterm::event::Event) -> EventResult {
        if self.handle.is_finished() {
            return Ok(Some(Message::ShowLoaded));
        }

        Ok(None)
    }
}
