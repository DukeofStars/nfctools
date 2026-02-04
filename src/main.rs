use std::{fs::OpenOptions, future::Future, path::PathBuf, sync::OnceLock};

use clap::{Parser, ValueEnum};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use dioxus::prelude::*;
use serde::Deserialize;
use tracing::{info, trace, warn, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

use crate::{fleet_data::FleetData, fleet_io::read_fleet};

mod fleet_data;
mod fleet_edit;
mod fleet_io;
mod load_fleets;
// mod tags;
mod test;
mod ui;

const NEBULOUS_GAME_ID_STEAM: u32 = 887570;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

fn spawn_async<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> impl Future<Output = T> {
    let (sender, receiver) = futures::channel::oneshot::channel();

    std::thread::spawn(move || {
        let res = f();
        // ignore error if receiver is dropped
        let _ = sender.send(res);
    });

    async move { receiver.await.expect("Thread panicked or sender dropped") }
}

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

fn default_cache_dir() -> PathBuf {
    let project_dirs = directories::ProjectDirs::from("", "", "NebTools")
        .expect("Unknown operating system");
    project_dirs.cache_dir().to_path_buf()
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct AppConfig {
    #[serde(default = "default_saves_dir")]
    saves_dir: PathBuf,
    #[serde(default)]
    excluded_dirs: Vec<String>,
    #[serde(default = "default_cache_dir")]
    cache_dir: PathBuf,
}

fn load_app_config() -> Result<()> {
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

    APP_CONFIG
        .set(app_config)
        .expect("load_app_config called twice");

    Ok(())
}

#[derive(Parser)]
#[clap(about, version)]
struct Cli {
    #[clap(short, long)]
    #[clap(default_value = "debug")]
    logging_level: LoggingLevel,
    #[clap(long)]
    test_fleets: bool,
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

    if cli.test_fleets {
        test::test_load_fleets()?;
        return Ok(());
    }

    info!("Starting NebTools");

    dioxus::launch(App);

    Ok(())
}

#[component]
fn App() -> Element {
    rsx! {
        FleetList {}
    }
}

#[component]
fn FleetList() -> Element {
    let fleets = use_resource(async move || {
        // Load app configuration first
        spawn_async(load_app_config).await.unwrap();
        // Then load fleets (load_fleets requires APP_CONFIG to be set)
        spawn_async(load_fleets::load_fleets).await
    });

    let mut selected_fleet_data = use_signal(|| None::<FleetData>);

    let selected_fleet = use_resource(move || async move {
        if let Some(fleet_data) = selected_fleet_data.as_ref() {
            let fleet_path = fleet_data.path.clone();
            let fleet = spawn_async(|| read_fleet(fleet_path));
            fleet.await.ok()
        } else {
            None
        }
    });

    rsx! {
        div {
            display: "grid",
            grid_template_columns: "33% 33% 33%",
            overflow: "hidden",
            height: "97vh",
            // Fleets List
            div {
                display: "flex",
                flex_direction: "column",
                min_height: 0,
                flex: 1,
                h2 { margin: 0, padding: 0, "Fleets" }
                div { overflow_y: "scroll", display: "grid",
                    // grid_template_columns: "",
                    match fleets.read().as_ref() {
                        Some(Ok(fleets)) => rsx! {
                            for fleet in fleets {
                                {
                                    let fleet = fleet.clone();
                                    rsx! {
                                        button {
                                            onclick: move |_| {
                                                println!("Selected fleet {}", fleet.name);
                                                selected_fleet_data.set(Some(fleet.clone()));
                                            },
                                            "{fleet.name}"
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => {
                            warn!("Failed to load fleets: {}", err);
                            rsx! {
                                div { "Failed to load fleets" }
                            }
                        }
                        None => rsx! {
                            div { "Loading fleets…" }
                        },
                    }
                }
            }
            // Fleet editor (middle)
            div { "Hello!!!" }
            div {
                display: "flex",
                flex_direction: "column",
                justify_content: "space-between",
                overflow: "hidden",
                h3 { "Hello, World!" }
                div {
                    h3 { "Ships" }
                    div { overflow_y: "scroll", display: "grid",
                        match selected_fleet.read().as_ref() {
                            Some(Some(fleet)) => rsx! {
                                for ship in fleet
                                    .ships
                                    .iter()
                                    .map(|ships| ships.ship.iter().map(|iter| iter.iter()))
                                    .flatten()
                                    .flatten()
                                {
                                    {
                                        rsx! {
                                            button { "{ship.name}" }
                                        }
                                    }
                                }
                            },
                            Some(None) => rsx! {
                                div { "Loading fleet..." }
                            },
                            None => rsx! {
                                div { "Loading fleet..." }
                            },
                        }
                    }
                }
            }
        }
    }
}
