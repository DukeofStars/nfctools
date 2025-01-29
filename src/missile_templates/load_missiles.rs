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
                .ok_or(my_error!("Invalid missile", "Missile has no designation"))?;

            output.push(MissileData {
                template_name: template_name.to_shared_string(),
                designation: designation.to_shared_string(),
                nickname: nickname.to_shared_string(),
                path: child.path().display().to_string().to_shared_string(),
                selected: false,
            });
        }
    }

    debug!("Loaded {} missiles", output.len());

    Ok(output)
}
// fn load_fleets_rec(
//     root_path: &PathBuf,
//     excluded_patterns: &Vec<Pattern>,
//     path: impl AsRef<Path>,
//     output: &mut Vec<FleetData>,
// ) -> Result<(), Error> {
//     let path = path.as_ref();
//     let mut children = path
//         .read_dir()
//         .map_err(|err| {
//             my_error!(
//                 format!("Failed to read directory '{}'", path.display()),
//                 err
//             )
//         })?
//         .filter_map(|c| c.ok())
//         .collect::<Vec<_>>();
//     children.sort_by(|a, b| {
//         if a.path().is_dir() {
//             Ordering::Greater
//         } else if b.path().is_dir() {
//             Ordering::Less
//         } else {
//             Ordering::Equal
//         }
//     });
//     'child_loop: for child in children {
//         let file_type = child
//             .file_type()
//             .wrap_err(format!(
//                 "Failed to determine filed type of '{}'",
//                 child.path().display()
//             ))
//             .map_err(|err| my_error!("Failed to determine file type", err))?;
//         if file_type.is_dir() {
//             load_fleets_rec(root_path, excluded_patterns, child.path(), output)?;
//         }
//         if file_type.is_file() {
//             if child.path().extension().map(|s| s.to_str()) != Some(Some("fleet".into())) {
//                 continue;
//             }
//             let fleet_info_reader =
//                 FleetInfoReader::new(File::open(child.path()).map_err(|err| {
//                     my_error!(
//                         format!("Failed to open file '{}'", child.path().display()),
//                         err
//                     )
//                 })?);
//             let fleet_name = fleet_info_reader.get_value("Fleet/Name");
//             for pattern in excluded_patterns {
//                 if pattern.matches_path(child.path().as_path()) {
//                     continue 'child_loop;
//                 }
//             }
//             let path = child.path();
//             let short_path = path
//                 .strip_prefix(root_path)
//                 .map_err(|err| {
//                     my_error!(
//                         format!("Failed to strip prefix from '{}'", path.display()),
//                         err
//                     )
//                 })?
//                 .parent()
//                 .map(|p| p.to_str().unwrap().to_string())
//                 .unwrap_or_default();
//             let fleet_data = FleetData {
//                 path: path.to_str().unwrap().into(),
//                 short_path: short_path.into(),
//                 selected: false,
//                 name: fleet_name.into(),
//             };
//             output.push(fleet_data);
//         }
//     }
//     Ok(())
// }
