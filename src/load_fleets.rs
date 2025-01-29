use std::{
    cmp::Ordering,
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use color_eyre::eyre::WrapErr;
use glob::Pattern;
use slint::{VecModel, Weak};
use tracing::debug;

use crate::{
    error::{wrap_errorable_function, Error},
    my_error, FleetData, FleetInfoReader, MainWindow,
};

pub fn on_reload_fleets_handler(
    main_window_weak: Weak<MainWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
    fleets_dir: PathBuf,
    excluded_patterns: Rc<Vec<Pattern>>,
) -> impl Fn() {
    move || {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            debug!("Reloading fleets list");
            let fleets = load_fleets(&fleets_dir, &excluded_patterns)
                .map_err(|err| my_error!("Failed to load fleets", err))?;
            fleets_model.set_vec(fleets);

            Ok(())
        });
    }
}

pub fn load_fleets(
    path: impl AsRef<Path>,
    excluded_patterns: &Vec<Pattern>,
) -> Result<Vec<FleetData>, Error> {
    debug!("Loading fleets from {}", path.as_ref().display());
    let mut output = vec![];
    load_fleets_rec(
        &path.as_ref().to_path_buf(),
        excluded_patterns,
        path,
        &mut output,
    )?;

    debug!("Loaded {} fleets", output.len());

    Ok(output)
}
fn load_fleets_rec(
    root_path: &PathBuf,
    excluded_patterns: &Vec<Pattern>,
    path: impl AsRef<Path>,
    output: &mut Vec<FleetData>,
) -> Result<(), Error> {
    let path = path.as_ref();
    let mut children = path
        .read_dir()
        .map_err(|err| {
            my_error!(
                format!("Failed to read directory '{}'", path.display()),
                err
            )
        })?
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
            .map_err(|err| my_error!("Failed to determine file type", err))?;
        if file_type.is_dir() {
            load_fleets_rec(root_path, excluded_patterns, child.path(), output)?;
        }
        if file_type.is_file() {
            if child.path().extension().map(|s| s.to_str()) != Some(Some("fleet".into())) {
                continue;
            }
            let fleet_info_reader =
                FleetInfoReader::new(File::open(child.path()).map_err(|err| {
                    my_error!(
                        format!("Failed to open file '{}'", child.path().display()),
                        err
                    )
                })?);
            let fleet_name = fleet_info_reader.get_value("Fleet/Name");
            for pattern in excluded_patterns {
                if pattern.matches_path(child.path().as_path()) {
                    continue 'child_loop;
                }
            }
            let path = child.path();
            let short_path = path
                .strip_prefix(root_path)
                .map_err(|err| {
                    my_error!(
                        format!("Failed to strip prefix from '{}'", path.display()),
                        err
                    )
                })?
                .parent()
                .map(|p| p.to_str().unwrap().to_string())
                .unwrap_or_default();
            let fleet_data = FleetData {
                path: path.to_str().unwrap().into(),
                short_path: short_path.into(),
                selected: false,
                name: fleet_name.into(),
            };
            output.push(fleet_data);
        }
    }
    Ok(())
}
