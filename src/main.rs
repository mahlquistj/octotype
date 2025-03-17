mod app;
mod config;
mod session;
mod utils;

use app::App;
use config::Config;
use directories::ProjectDirs;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config =
        Figment::from(Serialized::defaults(Config::default())).merge(Env::prefixed("TYPERS_"));

    if let Some(dirs) = ProjectDirs::from("com", "mahlquist", "Typers") {
        let config_dir = dirs.config_dir();
        let mut toml = config_dir.to_path_buf();
        toml.push("config.toml");

        if toml.exists() {
            config = config.merge(Toml::file(toml));
        }
    }
    let config = config.extract()?;

    let mut terminal = ratatui::init();
    let mut app = App::new(config);
    app.run(&mut terminal).expect("Crashed");
    ratatui::restore();

    Ok(())
}
