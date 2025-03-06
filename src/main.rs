use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Block,
    DefaultTerminal, Frame,
};
use smol_macros::main;

mod library;
mod session;
mod utils;

use library::Library;
use session::TypingSession;

main! {
    async fn main() {
        let mut terminal = ratatui::init();
        let result = run(&mut terminal).await;
        ratatui::restore();
        println!("{}", result.expect("crash"));
    }
}

pub async fn run(terminal: &mut DefaultTerminal) -> std::io::Result<String> {
    let mut session = Library::get_words(10, None).await.expect("Error");

    loop {
        terminal.draw(|frame| draw(frame, &mut session))?;
        if handle_events(&mut session)? || session.is_done() {
            break;
        }
    }
    let minutes = session
        .first_keypress
        .map(|inst| inst.elapsed().as_secs_f64())
        .unwrap_or_default()
        / 60.0;
    let characters = session.length() as f64;
    let wpm = (characters / 5.0) / minutes;
    Ok(format!("{wpm} Wpm"))
}


fn draw(frame: &mut Frame, session: &mut TypingSession) {
    let [title, content] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    frame.render_widget(Block::bordered().title("Typers"), title);

    session.render(frame, content).expect("SESSION ERROR");
}

fn handle_events(session: &mut TypingSession) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key)
            if key.kind == KeyEventKind::Press && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            if let KeyCode::Char('q') = key.code {
                return Ok(true);
            }
        }
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            // Add character
            KeyCode::Char(character) => session.add(character),
            // Delete character
            KeyCode::Backspace => {
                session.pop()
            }
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}
