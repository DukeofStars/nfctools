use std::{sync::Mutex, path::PathBuf, sync::OnceLock};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use serde::{Deserialize, Serialize};
use tracing::{info, trace, warn};

use crate::NEBULOUS_GAME_ID_STEAM;

pub static APP_CONFIG: OnceLock<Mutex<AppConfig>> = OnceLock::new();

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_saves_dir")]
    pub saves_dir: PathBuf,
    #[serde(default)]
    pub excluded_dirs: Vec<String>,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    #[serde(default = "default_true")]
    pub sound_effects: bool,
}

pub fn default_true() -> bool {
    true
}

pub fn load_app_config() -> Result<()> {
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
        .set(Mutex::new(app_config))
        .expect("load_app_config called twice");

    Ok(())
}

pub fn save_app_config() -> Result<()> {
    let config_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(
            eyre!("Failed to retrieve config dir")
                .wrap_err("OS not recognised?"),
        )?
        .preference_dir()
        .join("config.toml");
    trace!("Saving config to '{}'", config_path.display());
    
    let config = APP_CONFIG.get().unwrap().lock().unwrap();
    std::fs::write(&config_path, toml::to_string_pretty(&*config).wrap_err("Failed to serialize config")?).wrap_err("Failed to write config file")?;

    Ok(())
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
