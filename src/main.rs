#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, fs::OpenOptions, io::Write, path::PathBuf, rc::Rc};

use actions::save_description;
use clap::{Parser, ValueEnum};
use color_eyre::eyre::eyre;
use error::{wrap_errorable_function, Error};
use glob::Pattern;
use missile_templates::UsedMissilesCache;
use serde::Deserialize;
use slint::{CloseRequestResponse, ComponentHandle, Model, VecModel};
use tags::TagsRepository;
use tracing::{debug, info, trace, warn, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

use crate::{fleet_io::read_fleet, load_fleets::load_fleets};

mod actions;
mod error;
mod fleet_editor;
mod fleet_io;
mod load_fleets;
mod missile_templates;
mod scramble;
mod tags;
mod win_predictor;

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
        warn!(
            "Could not automatically detected nebulous installation directory, falling back to default. \
             This most likely means the app will fail"
        );
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
        .ok_or(my_error!("Failed to retrieve config dir", "OS not recognised?"))?
        .preference_dir()
        .join("config.toml");
    trace!("Loading config from '{}'", config_path.display());
    let config_file = std::fs::read_to_string(&config_path)
        .inspect_err(|_| trace!("No config file found, using default config values"))
        .unwrap_or_default();
    let app_config: AppConfig =
        toml::from_str(&config_file).map_err(|err| my_error!("Failed to parse config file", err))?;

    Ok(app_config)
}
fn load_tags() -> Result<TagsRepository, Error> {
    let tags_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(my_error!("Failed to retrieve config dir", "OS not recognised?"))?
        .preference_dir()
        .join("tags.toml");
    trace!("Loading tags from '{}'", tags_path.display());
    let tags_file = std::fs::read_to_string(&tags_path)
        .inspect_err(|_| trace!("No tags file found, using default config values"))
        .unwrap_or_default();
    let tags_repo: TagsRepository =
        toml::from_str(&tags_file).map_err(|err| my_error!("Failed to parse tags file", err))?;

    Ok(tags_repo)
}
fn save_tags(tags_repo: Rc<RefCell<TagsRepository>>) -> Result<(), Error> {
    debug!("Saving tags");
    let tags_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(my_error!("Failed to retrieve config dir", "OS not recognised?"))?
        .preference_dir()
        .join("tags.toml");
    trace!("Writing tags to '{}'", tags_path.display());
    let mut tags_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&tags_path)
        .map_err(|err| my_error!("Failed to open tags file", err))?;
    let toml =
        toml::to_string(tags_repo.as_ref()).map_err(|err| my_error!("Failed to serialize tags", err))?;
    tags_file
        .write_all(toml.as_bytes())
        .map_err(|err| my_error!("Failed to write tags file", err))?;

    Ok(())
}
fn load_missiles_cache() -> Result<UsedMissilesCache, Error> {
    debug!("Loading UsedMissilesCache");
    let missiles_cache_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(my_error!("Failed to retrieve cache dir", "OS not recognised?"))?
        .cache_dir()
        .join("missile_cache.toml");
    trace!(
        "Loading UsedMissilesCache from '{}'",
        missiles_cache_path.display()
    );
    let missiles_cache_text = std::fs::read_to_string(&missiles_cache_path)
        .map_err(|err| my_error!("Failed to read missiles cache file", err))?;
    let missiles_cache = toml::from_str(&missiles_cache_text)
        .map_err(|err| my_error!("Failed to deserialize UsedMissilesCache", err))?;

    Ok(missiles_cache)
}
fn save_missiles_cache(used_missiles_cache: &UsedMissilesCache) -> Result<(), Error> {
    debug!("Saving UsedMissilesCache");
    let missiles_cache_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(my_error!("Failed to retrieve cache dir", "OS not recognised?"))?
        .cache_dir()
        .join("missile_cache.toml");
    trace!("Writing UsedMissilesCache to '{}'", missiles_cache_path.display());
    if missiles_cache_path
        .parent()
        .is_some_and(|parent| !parent.exists())
    {
        debug!("Creating cache dir");
        std::fs::create_dir_all(&missiles_cache_path.parent().unwrap())
            .map_err(|err| my_error!("Failed to create cache dir", err))?;
    }

    if missiles_cache_path.exists() {
        trace!("Deleting old cache");
        std::fs::remove_file(&missiles_cache_path)
            .map_err(|err| my_error!("Failed to remove old cache", err))?;
    }

    let mut missiles_cache_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&missiles_cache_path)
        .map_err(|err| my_error!("Failed to open missiles cache file", err))?;
    let toml = toml::to_string(used_missiles_cache)
        .map_err(|err| my_error!("Failed to serialize UsedMissilesCache", err))?;
    missiles_cache_file
        .write_all(toml.as_bytes())
        .map_err(|err| my_error!("Failed to write UsedMissilesCache file", err))?;

    Ok(())
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

    // Initialise this first so it works even if an error occurs during configuration loading.
    main_window.on_close_without_saving(|| {
        warn!("Saving data failed, closing without saving");
        std::process::exit(1);
    });

    let (app_config, excluded_patterns, fleets_model) = wrap_errorable_function(&main_window, || {
        info!("Loading app configuration");
        let app_config = load_app_config()?;

        let excluded_patterns = app_config
            .excluded_dirs
            .iter()
            .map(|s| Pattern::new(s.as_str()).map_err(|err| my_error!("Failed to parse glob", err)))
            .collect::<Result<Vec<Pattern>, Error>>()?;

        let fleets = load_fleets(app_config.saves_dir.join("Fleets"), &excluded_patterns)?;

        let fleets_model = Rc::new(slint::VecModel::from(fleets));
        main_window.set_fleets(fleets_model.clone().into());
        debug!("Fleets passed to UI");

        // Generate UsedMissilesCache
        let used_missiles_cache = load_missiles_cache();
        let used_missiles_cache = if let Ok(mut used_missiles_cache) = used_missiles_cache {
            used_missiles_cache.update(&app_config.saves_dir.join("Fleets"), &excluded_patterns)?;

            used_missiles_cache
        } else {
            warn!(error = %used_missiles_cache.unwrap_err(), "Failed to load previous missile cache, generating a fresh cache");
            missile_templates::UsedMissilesCache::generate_from_fleets(&app_config.saves_dir.join("Fleets"), &excluded_patterns)?
        };
        save_missiles_cache(&used_missiles_cache)?;

        Ok((app_config, excluded_patterns, fleets_model))
    })
    .map_err(|err| eyre!("{}", err.error).wrap_err(err.title))
    .inspect_err(|_| {
        // run the main window to get the error screen.
        main_window.set_shutdown_state(true);
        let _ = main_window.run();
    })?;
    let excluded_patterns = Rc::new(excluded_patterns);

    debug!("Setting up callbacks");

    let tags_repo = Rc::new(RefCell::new(
        load_tags()
            .inspect_err(|err| warn!("{}", err))
            .unwrap_or_default(),
    ));
    let tags = Vec::new();
    let tags_model = Rc::new(VecModel::from(tags));
    main_window.set_tags(tags_model.clone().into());

    main_window.on_add_tag(tags::on_add_tag_handler(tags_model.clone(), tags_repo.clone()));

    main_window.on_remove_tag(tags::on_remove_tag_handler(tags_model.clone()));

    main_window.on_lookup_tag(tags::on_lookup_tag_handler(
        main_window.as_weak(),
        tags_repo.clone(),
    ));

    main_window.on_open_missiles_view(missile_templates::missiles_window::on_open_missiles_view_handler(
        main_window.as_weak(),
        app_config.saves_dir.join("MissileTemplates"),
        excluded_patterns.clone(),
    ));

    main_window.on_open_win_predictor(win_predictor::on_open_win_predictor_handler(
        main_window.as_weak(),
    ));

    main_window.on_open_fleet_editor(fleet_editor::on_open_fleet_editor_handler(
        main_window.as_weak(),
        fleets_model.clone(),
    ));

    main_window.on_merge(actions::merge::on_merge_handler(
        main_window.as_weak(),
        fleets_model.clone(),
    ));

    // main_window.on_save_description(actions::save_description::on_save_description_handler(
    //     main_window.as_weak(),
    //     fleets_model.clone(),
    //     tags_model.clone(),
    // ));

    main_window.on_reload_fleets(load_fleets::on_reload_fleets_handler(
        main_window.as_weak(),
        fleets_model.clone(),
        app_config.saves_dir.join("Fleets"),
        excluded_patterns.clone(),
    ));

    main_window.on_scramble_fleet(scramble::on_scramble_fleet_handler(
        main_window.as_weak(),
        fleets_model.clone(),
    ));

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        let tags_model = tags_model.clone();
        main_window.on_viewing(move |idx| {
            let main_window = main_window_weak.unwrap();
            let _ = wrap_errorable_function(&main_window_weak.unwrap(), || {
                let description = main_window.invoke_get_description().to_string();
                save_description::save_fleet_data(
                    &main_window,
                    fleets_model.clone(),
                    tags_model.clone(),
                    description,
                )?;

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

                let fleet_data = fleets_model.iter().nth(idx as usize).ok_or(my_error!(
                    "Selected fleet doesn't exist",
                    "cur_fleet_idx points to a nonexistant fleet"
                ))?;
                trace!("Viewing fleet {}: '{}'", idx, fleet_data.name);
                let fleet = read_fleet(&fleet_data.path)?;
                let description = &fleet.description;
                if description.as_ref().is_some_and(|d| !d.is_empty()) {
                    trace!(description = %description.as_ref().unwrap(), "Found description");
                }

                let (tags, description) =
                    tags::get_tags_from_description(description.as_ref().unwrap_or(&String::new()))?;
                main_window.invoke_set_description(description.into());
                tags_model.set_vec(tags);

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        let tags_model = tags_model.clone();
        let tags_repo = tags_repo.clone();
        main_window.window().on_close_requested(move || {
            let main_window = main_window_weak.unwrap();
            let res = wrap_errorable_function(&main_window_weak.unwrap(), || {
                let description = main_window.invoke_get_description().to_string();
                let res1 = save_description::save_fleet_data(
                    &main_window,
                    fleets_model.clone(),
                    tags_model.clone(),
                    description,
                );
                let res2 = save_tags(tags_repo.clone());

                // Batch the results together so both tags and fleet data have a chance to save.
                res1?;
                res2?;

                Ok(())
            });

            match res {
                Ok(_) => CloseRequestResponse::HideWindow,
                Err(_) => {
                    main_window.set_shutdown_state(true);
                    CloseRequestResponse::KeepWindowShown
                }
            }
        });
    }

    main_window.run()?;

    Ok(())
}
