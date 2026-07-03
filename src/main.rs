#![cfg_attr(feature = "bundle", windows_subsystem = "windows")]

use std::{fs::OpenOptions, path::PathBuf};

use clap::{Parser, ValueEnum};
use color_eyre::Result;
use dioxus::{
    desktop::{muda::Menu, wry::dpi::PhysicalSize, Config, WindowBuilder},
    prelude::*,
};
use lazy_static::lazy_static;
use tracing::{info, warn, Level};
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    layer::SubscriberExt,
    EnvFilter, Layer, Registry,
};

use crate::{menubar::Menubars, ui::fleet_list::FleetList};

mod audio;
mod config;
mod dressings;
mod export;
mod fleet_data;
mod fleet_edit;
mod fleet_io;
mod load_fleets;
mod menubar;
mod search;
mod spawn_async;
mod tags;
mod ui;

#[allow(unused)]
mod components;

const NEBULOUS_GAME_ID_STEAM: u32 = 887570;

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

lazy_static! {
    static ref LOG_FILE_PATH: Option<PathBuf> = std::env::current_exe()
        .map(|p| { p.parent().map(|p| { p.join("log.txt") }) })
        .ok()
        .flatten();
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    // Initialise logging
    let log_file = LOG_FILE_PATH.as_ref().map(|p| {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(p)
    });
    if let Some(Ok(file)) = log_file {
        let subscriber = Registry::default()
            .with(
                fmt::Layer::new()
                    .with_target(true)
                    .with_span_events(
                        tracing_subscriber::fmt::format::FmtSpan::NONE,
                    )
                    .with_filter(EnvFilter::new("info")),
            )
            .with(
                fmt::Layer::new()
                    .with_writer(file.with_max_level(Level::TRACE))
                    .with_target(true)
                    .with_span_events(
                        tracing_subscriber::fmt::format::FmtSpan::NONE,
                    )
                    .with_ansi(false)
                    .with_filter(EnvFilter::new(
                        "trace,warnings=debug,dioxus=debug",
                    )),
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
            Registry::default().with(fmt::Layer::new().map_writer(|w| {
                w.with_max_level(match cli.logging_level {
                    LoggingLevel::Full => Level::TRACE,
                    LoggingLevel::Debug => Level::DEBUG,
                    LoggingLevel::Info => Level::INFO,
                    LoggingLevel::None => Level::ERROR,
                })
            }));
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    // Launch app

    info!("Starting NebTools");

    let menubars = Menubars::new();

    let menu = Menu::new();
    menubars.attach_to_menu(&menu);

    crate::menubar::MENUBARS.set(Some(menubars));

    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! {
            Config::new().with_menu(Some(menu)).with_window(
                WindowBuilder::new()
                    .with_inner_size(PhysicalSize::new(1100, 600))
                    .with_title(format!("NebTools v{} @dukeofstars", env!("CARGO_PKG_VERSION")))
            )
        })
        .launch(App);

    Ok(())
}

#[macro_export]
macro_rules! include_style {
    ($path:literal) => {{
        #[component]
        fn ComponentStyle() -> Element {
            #[cfg(feature = "bundle")]
            rsx! {
                dioxus::document::Style {
                    {include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))}
                }
            }
            #[cfg(not(feature = "bundle"))]
            rsx! { dioxus::document::Stylesheet { href: asset!(concat!("/", $path)) } }
        }
        rsx! { ComponentStyle {} }
    }}
}

#[component]
fn App() -> Element {
    return rsx! {
        {include_style!("assets/dx-components-theme.css")}
        {include_style!("assets/main.css")}

        FleetList {}
    };
}
