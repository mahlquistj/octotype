use smol_macros::main;

mod app;
mod session;
mod utils;

use app::App;

main! {
    async fn main() {
        let mut terminal = ratatui::init();
        let mut app = App::new();
        let stats = app.run(&mut terminal).await.expect("Crashed");
        ratatui::restore();
        println!("{stats:#?}")
    }
}
