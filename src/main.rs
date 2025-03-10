pub mod app;
mod session;
mod utils;

use app::App;

fn main() {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal).expect("Crashed");
    ratatui::restore();
}
