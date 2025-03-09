mod app;
mod session;
mod utils;

use app::App;

fn main() {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let stats = app.run(&mut terminal).expect("Crashed");
    ratatui::restore();
    println!("{stats:#?}")
}
