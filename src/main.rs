mod app;
mod config;
mod session;
mod utils;

use app::App;
use config::Config;
use directories::ProjectDirs;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Grab default configuration
    let mut config = Figment::from(Serialized::defaults(Config::default()));

    // If toml config file exists, we want to merge the values into the config
    if let Some(dirs) = ProjectDirs::from("com", "mahlquist", "Typers") {
        let config_dir = dirs.config_dir();
        let mut toml = config_dir.to_path_buf();
        toml.push("config.toml");

        if toml.exists() {
            config = config.merge(Toml::file(toml));
        }
    }

    App::new(config.extract()?).run()?;

    Ok(())
}
