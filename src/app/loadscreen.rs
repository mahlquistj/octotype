use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};

use ratatui::{layout::Constraint, style::Style};
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse, BRAILLE_SIX};

use crate::utils::{center, Message, Page};

pub struct LoadingScreen {
    handle: JoinHandle<Result<Message, minreq::Error>>,
    state: ThrobberState,
    last_tick: Instant,
}

impl LoadingScreen {
    pub fn load(handle: JoinHandle<Result<Message, minreq::Error>>) -> Self {
        Self {
            handle,
            state: ThrobberState::default(),
            last_tick: Instant::now(),
        }
    }

    pub fn join(self) -> Message {
        return match self.handle.join() {
            Ok(Ok(msg)) => msg,
            Ok(Err(error)) => todo!("Show error screen: {error}"),
            Err(handle_error) => todo!("Show error scrren: {handle_error:?}"),
        };
    }

    fn tick(&mut self) {
        self.state.calc_next();
        self.last_tick = Instant::now();
    }
}

impl Page for LoadingScreen {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let throbber = Throbber::default()
            .label(format!("Loading words... {}", self.handle.is_finished()))
            .throbber_style(
                Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(BRAILLE_SIX)
            .use_type(WhichUse::Spin);

        frame.render_stateful_widget(throbber, center, &mut self.state);
    }

    fn poll(&mut self) -> Option<Message> {
        if self.last_tick.elapsed() > Duration::from_millis(200) {
            self.tick();
        }
        if self.handle.is_finished() {
            return Some(Message::ShowLoaded);
        }

        None
    }
}
