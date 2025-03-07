use smol_macros::main;

mod app;
mod library;
mod session;
mod text;
mod utils;

use app::App;

main! {
    async fn main() {
        let mut terminal = ratatui::init();
        let mut app = App::new();
        let wpm = app.run(&mut terminal).await.expect("Crashed");
        ratatui::restore();
        println!("wpm: {wpm}");
    }
}
