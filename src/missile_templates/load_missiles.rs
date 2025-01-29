use std::{
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use color_eyre::eyre::WrapErr;
use slint::{ToSharedString, VecModel, Weak};
use tracing::{debug, instrument, trace};
use xmltree::Element;

use crate::{
    error::{wrap_errorable_function_m, Error},
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
            if child.path().extension().map(|s| s.to_str()) != Some(Some("missile".into())) {
                continue;
            }

            trace!("Loading missile from '{}'", child.path().display());

            let element = {
                trace!("Opening missile file");
                let missile_file = File::open(child.path()).map_err(|err| {
                    my_error!(
                        format!("Failed to open missile '{}'", child.path().display()),
                        err
                    )
                })?;
                trace!("Parsing missile file");
                Element::parse(missile_file)
                    .map_err(|err| my_error!("Failed to parse missile file", err))?
            };

            let template_name = element
                .get_child("AssociatedTemplateName")
                .map(|elem| elem.get_text())
                .flatten()
                .unwrap_or_default();
            let designation = element
                .get_child("Designation")
                .map(|elem| elem.get_text())
                .flatten()
                .ok_or(my_error!("Invalid missile", "Missile has no designation"))?;
            let nickname = element
                .get_child("Nickname")
                .map(|elem| elem.get_text())
                .flatten()
                .ok_or(my_error!("Invalid missile", "Missile has no nickname"))?;
            let cost = element
                .get_child("Cost")
                .map(|elem| elem.get_text())
                .flatten()
                .map(|s| s.parse::<i32>().ok())
                .flatten()
                .ok_or(my_error!("Invalid missile", "Missile has no point cost"))?;

            output.push(MissileData {
                template_name: template_name.to_shared_string(),
                designation: designation.to_shared_string(),
                nickname: nickname.to_shared_string(),
                path: child.path().display().to_string().to_shared_string(),
                cost,
                selected: false,
            });
        }
    }

    debug!("Loaded {} missiles", output.len());

    Ok(output)
}
