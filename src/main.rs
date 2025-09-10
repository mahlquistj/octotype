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
    /// Prints the loaded settings
    #[arg(short, long)]
    print_settings: bool,

    /// Specifies a config location
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AppArgs::parse();

    let override_path = args
        .config
        .and_then(|dir| Some(PathBuf::from_str(&dir)))
        .transpose()?;

    let config = Config::get(override_path)?;

    if args.print_settings {
        println!("{}", toml::to_string_pretty(&config.settings)?);
        return Ok(());
    }

    App::new(config, session_factory).run()?;

    Ok(())
}
