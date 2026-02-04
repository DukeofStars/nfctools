use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::Hasher,
    path::{Path, PathBuf},
    str::FromStr,
};

use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use glob::Pattern;
use metrohash::MetroHash;
use tracing::{debug, info, warn};

use crate::{fleet_data::FleetData, fleet_io::read_fleet, APP_CONFIG};

pub fn load_fleets() -> Result<Vec<FleetData>> {
    let Some(app_config) = APP_CONFIG.get() else {
        bail!("App configuration not yet loaded");
    };

    let path = &app_config.saves_dir;
    let excluded_patterns = app_config
        .excluded_dirs
        .iter()
        .filter_map(|x| Pattern::from_str(x).ok())
        .collect::<Vec<_>>();

    let cache_path = app_config.cache_dir.join("fleets_data.bin");
    let get_fleet_cache = || {
        let bytes = std::fs::read(&cache_path).ok()?;
        let fleet_cache: HashMap<u64, FleetData> =
            postcard::from_bytes(&bytes).ok()?;
        info!("Loading fleets from cache");
        Some(fleet_cache)
    };
    let mut fleet_cache = get_fleet_cache().unwrap_or_default();

    debug!("Loading fleets from {}", path.display());
    let mut output = vec![];
    load_fleets_rec(
        path.as_ref(),
        &excluded_patterns,
        path.as_ref(),
        &mut output,
        &mut fleet_cache,
    )?;

    debug!("Loaded {} fleets", output.len());

    info!("Saving fleet cache");
    let bytes: Vec<u8> = postcard::to_stdvec(&fleet_cache).unwrap();
    if !app_config.cache_dir.exists() {
        let _ = std::fs::create_dir_all(&app_config.cache_dir);
    }
    if let Err(err) = std::fs::write(&cache_path, &bytes) {
        warn!(%err, "Failed to save fleets cache");
    }

    Ok(output)
}
fn load_fleets_rec(
    root_path: &Path,
    excluded_patterns: &Vec<Pattern>,
    path: &Path,
    output: &mut Vec<FleetData>,
    fleet_cache: &mut HashMap<u64, FleetData>,
) -> Result<()> {
    let mut children = path
        .read_dir()
        .wrap_err(format!("Failed to read directory '{}'", path.display()))?
        .filter_map(|c| c.ok())
        .collect::<Vec<_>>();
    children.sort_by(|a, b| {
        if a.path().is_dir() {
            Ordering::Greater
        } else if b.path().is_dir() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    'child_loop: for child in children {
        let file_type = child
            .file_type()
            .wrap_err(format!(
                "Failed to determine filed type of '{}'",
                child.path().display()
            ))
            .wrap_err("Failed to determine file type")?;
        if file_type.is_dir() {
            load_fleets_rec(
                root_path,
                excluded_patterns,
                &child.path(),
                output,
                fleet_cache,
            )?;
        }
        if file_type.is_file() {
            let path = child.path();
            for pattern in excluded_patterns {
                if pattern.matches_path(path.as_path()) {
                    continue 'child_loop;
                }
            }
            if path.extension().map(|s| s.to_str())
                != Some(Some("fleet".into()))
            {
                continue;
            }
            let Ok(hash) = hash_file(&path) else {
                warn!("Failed to hash fleet");
                continue 'child_loop;
            };
            let fleet_data = if let Some(fleet_data) = fleet_cache.get(&hash) {
                fleet_data.clone()
            } else {
                let fleet = match read_fleet(&path) {
                    Ok(fleet) => fleet,
                    Err(err) => {
                        warn!(
                            "Skipping invalid fleet '{}': {}",
                            path.display(),
                            err
                        );
                        continue 'child_loop;
                    }
                };
                let short_path = path
                    .strip_prefix(root_path)
                    .wrap_err(format!(
                        "Failed to strip prefix from '{}'",
                        path.display()
                    ))?
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or(PathBuf::new());
                let fleet_data = FleetData {
                    path,
                    short_path: short_path.into(),
                    selected: false,
                    name: fleet.name.into(),
                };

                fleet_cache.insert(hash, fleet_data.clone());

                fleet_data
            };

            output.push(fleet_data);
        }
    }
    Ok(())
}

pub fn hash_file(path: impl AsRef<Path>) -> Result<u64> {
    let mut hasher = MetroHash::new();
    let bytes = std::fs::read(path)?;
    hasher.write(&bytes);
    Ok(hasher.finish())
}
