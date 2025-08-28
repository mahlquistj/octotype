mod app;
mod config;
mod page;
mod sources;
mod utils;

use std::{path::PathBuf, str::FromStr};

use app::App;
use clap::Parser;
use config::Config;
use directories::ProjectDirs;
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};

/// Cli-Arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct AppArgs {
    /// Prints the current configuration
    #[arg(short, long)]
    print_config: bool,

    /// Specifies a config location
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AppArgs::parse();

    // Grab default configuration
    let mut config = Figment::from(Serialized::defaults(Config::default()));

    // Check for toml file location
    let toml = if let Some(custom_path) = args.config {
        PathBuf::from_str(&custom_path)?
    } else if let Some(dirs) = ProjectDirs::from("com", "Mahlquist", "OctoType") {
        let config_dir = dirs.config_dir();
        let mut toml = config_dir.to_path_buf();
        toml.push("config.toml");
        toml
    } else {
        PathBuf::new()
    };

    if toml.exists() {
        config = config.merge(Toml::file(toml));
    }

    let config = config.extract()?;

    if args.print_config {
        println!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    App::new(config).run()?;

    Ok(())
}
