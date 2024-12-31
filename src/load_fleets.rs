use std::{
    cmp::Ordering,
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use slint::{VecModel, Weak};
use tracing::debug;

use crate::{error::wrap_errorable_function, my_error, FleetData, FleetInfoReader, MainWindow};

pub fn on_reload_fleets_handler(
    main_window_weak: Weak<MainWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
    fleets_dir: PathBuf,
) -> impl Fn() {
    move || {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            debug!("Reloading fleets list");
            let fleets =
                load_fleets(&fleets_dir).map_err(|err| my_error!("Failed to load fleets", err))?;
            fleets_model.set_vec(fleets);

            Ok(())
        });
    }
}

pub fn load_fleets(path: impl AsRef<Path>) -> color_eyre::Result<Vec<FleetData>> {
    debug!("Loading fleets from {}", path.as_ref().display());
    let mut output = vec![];
    load_fleets_rec(&path.as_ref().to_path_buf(), path, &mut output)?;

    debug!("Loaded {} fleets", output.len());

    Ok(output)
}
fn load_fleets_rec(
    root_path: &PathBuf,
    path: impl AsRef<Path>,
    output: &mut Vec<FleetData>,
) -> color_eyre::Result<()> {
    let path = path.as_ref();
    let mut children = path.read_dir()?.filter_map(|c| c.ok()).collect::<Vec<_>>();
    children.sort_by(|a, b| {
        if a.path().is_dir() {
            Ordering::Greater
        } else if b.path().is_dir() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    for child in children {
        let file_type = child.file_type()?;
        if file_type.is_dir() {
            load_fleets_rec(root_path, child.path(), output)?;
        }
        if file_type.is_file() {
            if child.path().extension().map(|s| s.to_str()) != Some(Some("fleet".into())) {
                continue;
            }
            let fleet_info_reader = FleetInfoReader::new(File::open(child.path())?);
            let fleet_name = fleet_info_reader.get_value("Fleet/Name");
            let path = child.path().to_path_buf();
            let short_path = path
                .strip_prefix(root_path)?
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
