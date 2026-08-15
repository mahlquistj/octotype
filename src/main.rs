mod app;
mod config;
mod page;
mod statistics;
mod utils;

use std::{path::PathBuf, str::FromStr};

use app::App;
use clap::Parser;

use crate::config::Config;

/// Cli-Arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AppArgs {
    /// Prints the loaded config (Settings, Modes, Sources)
    #[arg(long)]
    pub print_config: bool,

    /// Prints the loaded settings
    #[arg(short, long)]
    pub print_settings: bool,

    /// Specifies a config location
    #[arg(short, long)]
    pub config: Option<String>,

    /// Specifies a mode to run
    #[arg(short, long)]
    pub mode: Option<String>,

    /// Specifies a source to run
    #[arg(short, long, requires = "mode")]
    pub source: Option<String>,

    /// Specifies a parameter in the format 'name:value'
    #[arg(
        short = 'P',
        long = "param",
        value_name = "PARAM:VALUE",
        requires = "source"
    )]
    pub params: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AppArgs::parse();

    let override_path = args
        .config
        .as_ref()
        .map(|dir| PathBuf::from_str(dir))
        .transpose()?;

    let config = Config::get(override_path)?;

    if args.print_config {
        println!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    if args.print_settings {
        println!("{}", toml::to_string_pretty(&config.settings)?);
        return Ok(());
    }

    App::new(config, args)?.run()?;

    Ok(())
}
