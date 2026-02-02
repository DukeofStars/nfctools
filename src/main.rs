use std::{fs::OpenOptions, future::Future, path::PathBuf, str::FromStr};

use clap::{Parser, ValueEnum};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use dioxus::prelude::*;
use glob::Pattern;
use serde::Deserialize;
use tracing::{info, trace, warn, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

use crate::fleet_data::FleetData;

mod fleet_data;
mod fleet_edit;
mod fleet_io;
mod load_fleets;
// mod tags;
mod ui;

const NEBULOUS_GAME_ID_STEAM: u32 = 887570;

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

#[derive(Deserialize)]
#[allow(unused)]
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

    info!("Starting NebTool");

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
        let config = spawn_async(load_app_config).await.unwrap();

        spawn_async(move || {
            load_fleets::load_fleets(
                config.saves_dir,
                &config
                    .excluded_dirs
                    .iter()
                    .map(|path| Pattern::from_str(&path).unwrap())
                    .collect::<Vec<_>>(),
            )
        })
        .await
    });

    let mut selected_fleet_data = use_signal(|| None::<FleetData>);

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
                        Some(Err(_)) => rsx! {
                            div { "Failed to load fleets" }
                        },
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
                h1 { "Hello, World!" }
            }
        }
    }
}
