#![cfg_attr(feature = "bundle", windows_subsystem = "windows")]

use std::{fs::OpenOptions, path::PathBuf};

use clap::Parser;
use color_eyre::{eyre::Context, Result};
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
    #[clap(default_value = "info")]
    logging_filter: String,
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
    let console_layer = tracing_subscriber::fmt::Layer::new()
        .with_target(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_filter(EnvFilter::new(cli.logging_filter));
    let file_layer = if let Some(Ok(file)) = log_file {
        Some(
            fmt::Layer::new()
                .with_writer(file.with_max_level(Level::TRACE))
                .with_target(true)
                .with_span_events(
                    tracing_subscriber::fmt::format::FmtSpan::NONE,
                )
                .with_ansi(false), // .with_filter(EnvFilter::new(
                                   //     "trace,warnings=debug,dioxus=debug",
                                   // )),
        )
    } else {
        None
    };
    let subscriber = Registry::default().with(console_layer).with(file_layer);
    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("Failed to initialise logger")?;

    std::thread::spawn(|| {
        if let Err(err) = update() {
            warn!("Self update failed: {:?}", err);
        }
    });

    debug!("Initialising clipboard handler");
    // Some platforms require clipboard to be kept alive for the lifetime of the program.
    let _clipboard =
        arboard::Clipboard::new().wrap_err("Failed to create clipboard")?;

    // Launch app

    info!("Starting NebTools");

    debug!("Initialising system menu");
    let menubars = Menubars::new();
    let menu = Menu::new();
    menubars.attach_to_menu(&menu);
    crate::menubar::MENUBARS.set(Some(menubars));

    debug!("Launching app");
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! {
            Config::new()
                .with_menu(Some(menu))
                .with_window(
                    WindowBuilder::new()
                        .with_inner_size(PhysicalSize::new(1100, 600))
                        .with_title(format!(
                            "NebTools v{} @dukeofstars",
                            env!("CARGO_PKG_VERSION")
                        ))
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
    let font = rsx! {
        dioxus::document::Style {
            {format!(r#"
                @font-face {{
                    font-family: "Bombardier";
                    src: url("data:font/ttf;base64,{}") format("truetype");
                    font-weight: normal;
                    font-style: normal;
                }}
            "#, include_str!(concat!(env!("OUT_DIR"), "/bombardier.ttf.base64")))}
        }
    };

    return rsx! {
        {font}

        {include_style!("assets/dx-components-theme.css")}
        {include_style!("assets/main.css")}

        FleetList {}
    };
}

#[cfg(feature = "auto-update")]
fn update() -> Result<()> {
    info!("Checking for app updates!");
    let status = self_update::backends::github::Update::configure()
        .repo_owner("DukeofStars")
        .repo_name("nfctools")
        .bin_name("nfctools")
        .show_download_progress(true)
        .current_version(self_update::cargo_crate_version!())
        .build()?
        .update()?;
    match status {
        self_update::Status::UpToDate(_) => info!("NebTools up to date!"),
        self_update::Status::Updated(ver) => {
            info!("NebTools updated to `{ver}`")
        }
    }
    Ok(())
}
#[cfg(not(feature = "auto-update"))]
fn update() -> Result<()> {
    Ok(())
}
