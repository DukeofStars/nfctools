use std::{fs::File, path::PathBuf};

use color_eyre::eyre::OptionExt;
use error::wrap_errorable_function;
use fleet_info_reader::FleetInfoReader;
use serde::Deserialize;
use slint::{ComponentHandle, Model};
use tracing::{debug, info, level_filters::LevelFilter, trace};

use crate::load_fleets::load_fleets;

mod actions;
mod error;
mod fleet_editor;
mod fleet_info_reader;
mod load_fleets;

slint::include_modules!();

fn default_saves_dir() -> PathBuf {
    PathBuf::from(r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\"#)
}

#[derive(Deserialize)]
struct AppConfig {
    #[serde(default = "default_saves_dir")]
    saves_dir: PathBuf,
}

fn load_app_config() -> color_eyre::Result<AppConfig> {
    let config_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or_eyre("Failed to retrieve config dir")?
        .preference_dir()
        .join("config.toml");
    trace!("Loading config from '{}'", config_path.display());
    let config_file = std::fs::read_to_string(&config_path)
        .inspect_err(|_| trace!("No config file found, using default config values"))
        .unwrap_or_default();
    let app_config: AppConfig = toml::from_str(&config_file)?;

    Ok(app_config)
}

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();
    color_eyre::install()?;

    info!("Starting NebTools");

    let main_window = MainWindow::new()?;

    debug!("Loading app configuration");
    let app_config = load_app_config()?;

    let fleets = load_fleets(app_config.saves_dir.join("Fleets"))?;

    let fleets_model = std::rc::Rc::new(slint::VecModel::from(fleets));
    main_window.set_fleets(fleets_model.clone().into());
    debug!("Fleets passed to UI");

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
                trace!(%description, "Found description");

                main_window.invoke_update_description(description.into());

                Ok(())
            });
        });
    }

    main_window.run()?;

    Ok(())
}
