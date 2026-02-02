use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::Context, Result};
use glob::Pattern;
use tracing::{debug, warn};

use crate::{fleet_data::FleetData, fleet_io::read_fleet};

pub fn load_fleets(
    path: impl AsRef<Path>,
    excluded_patterns: &Vec<Pattern>,
) -> Result<Vec<FleetData>> {
    debug!("Loading fleets from {}", path.as_ref().display());
    let mut output = vec![];
    load_fleets_rec(
        path.as_ref(),
        excluded_patterns,
        path.as_ref(),
        &mut output,
    )?;

    debug!("Loaded {} fleets", output.len());

    Ok(output)
}
fn load_fleets_rec(
    root_path: &Path,
    excluded_patterns: &Vec<Pattern>,
    path: &Path,
    output: &mut Vec<FleetData>,
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
            output.push(fleet_data);
        }
    }
    Ok(())
}
