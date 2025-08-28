mod app;
mod config;
mod modes;
mod page;
mod session_factory;
mod sources;
mod utils;

use std::{path::PathBuf, str::FromStr};

use app::App;
use clap::Parser;
use config::Config;
use directories::ProjectDirs;
use sources::SourceManager;
use modes::ModeManager;
use session_factory::SessionFactory;
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

    // Get config directory
    let config_dir = if let Some(dirs) = ProjectDirs::from("com", "Mahlquist", "OctoType") {
        dirs.config_dir().to_path_buf()
    } else {
        PathBuf::from(".")
    };
    
    // Load sources and modes
    let source_manager = SourceManager::load_from_config_dir(&config_dir)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load sources: {}", e);
            SourceManager::with_defaults()
        });
        
    let mode_manager = ModeManager::load_from_config_dir(&config_dir)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load modes: {}", e);
            ModeManager::with_defaults()
        });
    
    let session_factory = SessionFactory::new(source_manager, mode_manager);
    
    App::new(config, session_factory).run()?;

    Ok(())
}
