// On windows, disable logging to stdout (which will open a terminal window) when built in release mode.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use color_eyre::eyre::eyre;
use error::{wrap_errorable_function, Error};
use fleet_info_reader::FleetInfoReader;
use glob::Pattern;
use serde::Deserialize;
use slint::{ComponentHandle, Model};
use tracing::{debug, info, trace, warn, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

use crate::load_fleets::load_fleets;

mod actions;
mod error;
mod fleet_editor;
mod fleet_info_reader;
mod load_fleets;

slint::include_modules!();

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
        warn!("Could not automatically detected nebulous installation directory, falling back to default. This most likely means the app will fail");
        PathBuf::from(r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\"#)
    }
}

#[derive(Deserialize)]
struct AppConfig {
    #[serde(default = "default_saves_dir")]
    saves_dir: PathBuf,
    #[serde(default)]
    excluded_dirs: Vec<String>,
}

fn load_app_config() -> Result<AppConfig, Error> {
    let config_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(my_error!(
            "Failed to retrieve config dir",
            "OS not recognised?"
        ))?
        .preference_dir()
        .join("config.toml");
    trace!("Loading config from '{}'", config_path.display());
    let config_file = std::fs::read_to_string(&config_path)
        .inspect_err(|_| trace!("No config file found, using default config values"))
        .unwrap_or_default();
    let app_config: AppConfig = toml::from_str(&config_file)
        .map_err(|err| my_error!("Failed to parse config file", err))?;

    Ok(app_config)
}

#[derive(Parser)]
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

fn main() -> color_eyre::Result<()> {
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
        let subscriber = Registry::default().with(Layer::new().map_writer(|w| {
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

    let main_window = MainWindow::new()?;
    let (app_config, excluded_patterns, fleets_model) =
        wrap_errorable_function(&main_window, || {
            info!("Loading app configuration");
            let app_config = load_app_config()?;

            let excluded_patterns = app_config
                .excluded_dirs
                .iter()
                .map(|s| {
                    Pattern::new(s.as_str()).map_err(|err| my_error!("Failed to parse glob", err))
                })
                .collect::<Result<Vec<Pattern>, Error>>()?;

            let fleets = load_fleets(app_config.saves_dir.join("Fleets"), &excluded_patterns)?;

            let fleets_model = std::rc::Rc::new(slint::VecModel::from(fleets));
            main_window.set_fleets(fleets_model.clone().into());
            debug!("Fleets passed to UI");

            Ok((app_config, excluded_patterns, fleets_model))
        })
        .map_err(|err| eyre!("{}", err.error).wrap_err(err.title))
        .inspect_err(|_| {
            // run the main window to get the error screen.
            let _ = main_window.run();
        })?;

    debug!("Setting up callbacks");

    main_window.on_open_fleet_editor(fleet_editor::on_open_fleet_editor_handler(
        main_window.as_weak(),
        fleets_model.clone(),
    ));

    main_window.on_merge(actions::merge::on_merge_handler(
        main_window.as_weak(),
        fleets_model.clone(),
    ));

    main_window.on_save_description(actions::save_description::on_save_description_handler(
        main_window.as_weak(),
        fleets_model.clone(),
    ));

    main_window.on_reload_fleets(load_fleets::on_reload_fleets_handler(
        main_window.as_weak(),
        fleets_model.clone(),
        app_config.saves_dir.join("Fleets"),
        excluded_patterns,
    ));

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_viewing(move |idx| {
            let main_window = main_window_weak.unwrap();
            let _ = wrap_errorable_function(&main_window_weak.unwrap(), || {
                if !main_window.get_multi_selecting() {
                    fleets_model.set_vec(
                        fleets_model
                            .iter()
                            .enumerate()
                            .map(|(f_idx, mut fleet)| {
                                if f_idx as i32 != idx {
                                    fleet.selected = false;
                                }
                                fleet
                            })
                            .collect::<Vec<_>>(),
                    );
                }

                // idx is set to -1 when a fleet is unselected, meaning no fleet is selected.
                if idx == -1 {
                    return Ok(());
                }

                let fleet = fleets_model.iter().nth(idx as usize).ok_or(my_error!(
                    "Selected fleet doesn't exist",
                    "cur_fleet_idx points to a nonexistant fleet"
                ))?;
                trace!("Viewing fleet {}: '{}'", idx, fleet.name);
                let fleet_info_reader =
                    FleetInfoReader::new(File::open(fleet.path.to_string()).map_err(|err| {
                        my_error!(
                            format!("Failed to open fleet '{}'", fleet.path.to_string()),
                            err
                        )
                    })?);
                let description = fleet_info_reader.get_value("Fleet/Description");
                if !description.is_empty() {
                    trace!(%description, "Found description");
                }

                main_window.invoke_update_description(description.into());

                Ok(())
            });
        });
    }

    main_window.run()?;

    Ok(())
}
