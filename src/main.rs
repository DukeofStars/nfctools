use std::fs::OpenOptions;

use clap::{Parser, ValueEnum};
use color_eyre::Result;
use dioxus::prelude::*;
use tracing::{info, warn, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

use crate::ui::fleet_list::FleetList;

mod fleet_data;
// mod fleet_edit;
mod fleet_io;
mod load_fleets;
// mod tags;
mod config;
mod spawn_async;
mod test;
mod ui;

const NEBULOUS_GAME_ID_STEAM: u32 = 887570;

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

    // Initialise logging
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

    // Launch app

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
