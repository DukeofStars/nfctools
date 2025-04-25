use std::{fs::OpenOptions, path::PathBuf};

use clap::{Parser, ValueEnum};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use serde::Deserialize;
use tracing::{info, trace, warn, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

mod fleet_data;
mod fleet_io;
mod load_fleets;
mod tags;
mod ui;

const NEBULOUS_GAME_ID_STEAM: u32 = 887570;

fn default_saves_dir() -> PathBuf {
    if let Ok(Ok(Some(path))) = steamlocate::SteamDir::locate()
        .inspect_err(|err| warn!(%err, "Could not locate steam directory"))
        .map(|steam_dir| {
            steam_dir
                .find_app(NEBULOUS_GAME_ID_STEAM)
                .inspect_err(|err| warn!(%err, "Could not search for app"))
                .map(|r| match r {
                    Some((app, lib)) => Some(lib.resolve_app_dir(&app)),
                    None => {
                        warn!("Nebulous installation was not found");
                        None
                    }
                })
        })
    {
        info!(
            "Automatically detected nebulous install directory at '{}'",
            path.display()
        );
        path.join("Saves")
    } else {
        warn!(
            "Could not automatically detected nebulous installation \
             directory, falling back to default. This most likely means the \
             app will fail"
        );
        PathBuf::from(
            r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\"#,
        )
    }
}

#[derive(Deserialize)]
struct AppConfig {
    #[serde(default = "default_saves_dir")]
    saves_dir: PathBuf,
    #[serde(default)]
    excluded_dirs: Vec<String>,
}

fn load_app_config() -> Result<AppConfig> {
    let config_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(
            eyre!("Failed to retrieve config dir")
                .wrap_err("OS not recognised?"),
        )?
        .preference_dir()
        .join("config.toml");
    trace!("Loading config from '{}'", config_path.display());
    let config_file = std::fs::read_to_string(&config_path)
        .inspect_err(|_| {
            trace!("No config file found, using default config values")
        })
        .unwrap_or_default();
    let app_config: AppConfig =
        toml::from_str(&config_file).wrap_err("Failed to parse config file")?;

    Ok(app_config)
}

#[derive(Parser)]
#[clap(about, version)]
struct Cli {
    #[clap(short, long)]
    #[clap(default_value = "debug")]
    logging_level: LoggingLevel,
}
#[derive(Clone, ValueEnum)]
enum LoggingLevel {
    Full,
    Debug,
    Info,
    None,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let log_file = std::env::current_exe().map(|p| {
        p.parent().map(|p| {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(p.join("log.txt"))
        })
    });

    if let Ok(Some(Ok(file))) = log_file {
        let subscriber = Registry::default()
            .with(Layer::new().map_writer(|w| {
                w.with_max_level(match cli.logging_level {
                    LoggingLevel::Full => Level::TRACE,
                    LoggingLevel::Debug => Level::DEBUG,
                    LoggingLevel::Info => Level::INFO,
                    LoggingLevel::None => Level::ERROR,
                })
            }))
            .with(
                Layer::new()
                    .with_writer(file.with_max_level(Level::TRACE))
                    .with_ansi(false),
            );
        tracing::subscriber::set_global_default(subscriber).unwrap();
        info!(
            "Writing logs to '{}'",
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("log.txt")
                .display()
        );
    } else {
        warn!("Could not open log file. Only printing logs to stdout now.");
        let subscriber =
            Registry::default().with(Layer::new().map_writer(|w| {
                w.with_max_level(match cli.logging_level {
                    LoggingLevel::Full => Level::TRACE,
                    LoggingLevel::Debug => Level::DEBUG,
                    LoggingLevel::Info => Level::INFO,
                    LoggingLevel::None => Level::ERROR,
                })
            }));
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    info!("Starting NebTools");

    let app_config = load_app_config()?;

    ui::launch(&app_config)?;

    Ok(())
}
