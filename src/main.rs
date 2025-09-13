mod app;
mod config;
mod page;
mod utils;

use std::{path::PathBuf, str::FromStr};

use app::App;
use clap::Parser;

use crate::config::Config;

/// Cli-Arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct AppArgs {
    /// Prints the loaded config (Settings, Modes, Sources)
    #[arg(long)]
    print_config: bool,

    /// Prints the loaded settings
    #[arg(short, long)]
    print_settings: bool,

    /// Specifies a config location
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AppArgs::parse();

    let override_path = args.config.map(|dir| PathBuf::from_str(&dir)).transpose()?;

    let config = Config::get(override_path)?;

    if args.print_config {
        println!("# SETTINGS\n{}", toml::to_string_pretty(&config.settings)?);

        println!("# MODES");
        for (name, mode) in &config.modes {
            println!("## {name}\n{}", toml::to_string_pretty(&dbg!(mode))?);
        }

        println!("# SOURCES");
        for (name, source) in &config.sources {
            println!("## {name}\n{}", toml::to_string_pretty(&source)?)
        }

        return Ok(());
    }

    if args.print_settings {
        println!("{}", toml::to_string_pretty(&config.settings)?);
        return Ok(());
    }

    App::new(config).run()?;

    Ok(())
}
