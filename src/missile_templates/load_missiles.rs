use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use color_eyre::eyre::WrapErr;
use slint::{ToSharedString, VecModel, Weak};
use tracing::{debug, instrument, trace};

use crate::{
    error::{wrap_errorable_function_m, Error},
    fleet_io::read_missile,
    my_error, MissileData, MissileWindow,
};

pub fn on_reload_missiles_handler(
    window_weak: Weak<MissileWindow>,
    missiles_model: Rc<VecModel<MissileData>>,
    missiles_dir: PathBuf,
) -> impl Fn() {
    move || {
        let window = window_weak.unwrap();
        let _ = wrap_errorable_function_m(&window, || {
            debug!("Reloading fleets list");
            let missiles = load_missiles(&missiles_dir)
                .map_err(|err| my_error!("Failed to load fleets", err))?;
            missiles_model.set_vec(missiles);

            Ok(())
        });
    }
}

#[instrument(skip(path))]
pub fn load_missiles(path: impl AsRef<Path>) -> Result<Vec<MissileData>, Error> {
    let path = path.as_ref();

    debug!("Loading missiles from {}", path.display());
    let mut output = vec![];

    let children = path
        .read_dir()
        .map_err(|err| {
            my_error!(
                format!("Failed to read directory '{}'", path.display()),
                err
            )
        })?
        .filter_map(|c| c.ok());
    for child in children {
        let file_type = child
            .file_type()
            .wrap_err(format!(
                "Failed to determine filed type of '{}'",
                child.path().display()
            ))
            .map_err(|err| my_error!("Failed to determine file type", err))?;

        if file_type.is_file() {
            let path = child.path();
            if path.extension().map(|s| s.to_str()) != Some(Some("missile".into())) {
                continue;
            }

            trace!("Loading missile from '{}'", path.display());

            let missile = read_missile(&path)?;

            output.push(MissileData {
                template_name: missile
                    .associated_template_name
                    .unwrap_or_default()
                    .to_shared_string(),
                designation: missile.designation.to_shared_string(),
                nickname: missile.nickname.to_shared_string(),
                path: child.path().display().to_string().to_shared_string(),
                cost: missile.cost.parse::<i32>().map_err(|err| {
                    my_error!(
                        "Invalid fleet file",
                        format!("Failed to parse cost: {}", err)
                    )
                })?,
                selected: false,
            });
        }
    }

    debug!("Loaded {} missiles", output.len());

    Ok(output)
}
