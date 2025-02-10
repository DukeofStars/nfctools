use std::{path::PathBuf, rc::Rc};

use glob::Pattern;
use slint::{ComponentHandle, Model, SharedString, ToSharedString, VecModel, Weak};
use tracing::{debug, info};

use crate::{
    error::{wrap_errorable_function_m, Error},
    fleet_io::{read_fleet, read_missile, write_fleet},
    missile_templates::MissileTemplateId,
    my_error, MissileData, MissileWindow, UpdateMissilesConfirmDialog,
};

pub fn on_update_fleets_with_missile_handler(
    missiles_dir: PathBuf,
    excluded_patterns: Rc<Vec<Pattern>>,
    window_weak: Weak<MissileWindow>,
    missiles_model: Rc<VecModel<MissileData>>,
) -> impl Fn(i32) {
    move |missile_idx: i32| {
        let window = window_weak.unwrap();
        let _ = wrap_errorable_function_m(&window, || {
            if missile_idx < 0 {
                return Err(my_error!("No missile selected", ""));
            }
            let missile_data = missiles_model.iter().nth(missile_idx as usize).unwrap();

            if missile_data.template_name == "" {
                return Err(my_error!("Missile has no associated template", ""));
            }

            let mut used_missiles_cache = crate::load_missiles_cache()
                .map_err(|err| my_error!("Could not load missiles cache", err))?;
            used_missiles_cache.update(&missiles_dir, &excluded_patterns)?;
            crate::save_missiles_cache(&used_missiles_cache)
                .map_err(|err| my_error!("Failed to save missiles cache", err))?;

            let fleets_that_use_missile = used_missiles_cache
                .fleets
                .into_iter()
                .filter(|(_path, used_missiles)| {
                    used_missiles
                        .used_missiles
                        .iter()
                        .any(|a| *a.0 == *missile_data.template_name)
                })
                .collect::<Vec<_>>();

            debug!("Opening confirmation dialog");
            let confirm_dialog = UpdateMissilesConfirmDialog::new()
                .map_err(|err| my_error!("Failed to create confirm dialog window", err))?;

            let fleet_names = fleets_that_use_missile
                .iter()
                .map(|(_, f)| Ok(f.name.to_shared_string()))
                .collect::<Result<Vec<_>, Error>>()?;
            let fleet_names_model = Rc::new(VecModel::from(fleet_names));
            confirm_dialog.set_fleet_names(fleet_names_model.clone().into());

            confirm_dialog.set_confirmed_fleets(Rc::new(VecModel::from(Vec::new())).into());

            {
                let window_weak = confirm_dialog.as_weak();
                confirm_dialog.on_confirm_fleet(move |fleet_name| {
                    let window = window_weak.unwrap();
                    let confirmed_fleets = window.get_confirmed_fleets();
                    let confirmed_fleets = confirmed_fleets
                        .as_any()
                        .downcast_ref::<VecModel<SharedString>>()
                        .expect("We know we set a VecModel earlier");
                    confirmed_fleets.push(fleet_name);
                });
            }

            {
                let window_weak = confirm_dialog.as_weak();
                confirm_dialog.on_unconfirm_fleet(move |fleet_name| {
                    let window = window_weak.unwrap();
                    let confirmed_fleets = window.get_confirmed_fleets();
                    let confirmed_fleets = confirmed_fleets
                        .as_any()
                        .downcast_ref::<VecModel<SharedString>>()
                        .expect("We know we set a VecModel earlier");
                    let idx = confirmed_fleets
                        .iter()
                        .position(|name| name == fleet_name)
                        .unwrap();
                    confirmed_fleets.remove(idx);
                });
            }

            {
                let window_weak = window.as_weak();
                confirm_dialog.on_confirmed_update_fleets(move |fleet_names| {
                    info!(
                        "Updating template {} in {} fleets: {:?}",
                        missile_data.template_name,
                        fleet_names.iter().count(),
                        fleet_names.iter().collect::<Vec<_>>()
                    );
                    let window = window_weak.unwrap();
                    let _ = wrap_errorable_function_m(&window, || {
                        let fleets = fleets_that_use_missile
                            .iter()
                            .filter(|(_, f)| fleet_names.iter().any(|name| *name == *f.name))
                            .collect::<Vec<_>>();

                        let new_missile = read_missile(&missile_data.path)?;

                        for (fleet_path, _) in &fleets {
                            let mut fleet = read_fleet(fleet_path)?;

                            let missile_types = fleet.missile_types.as_mut().ok_or(my_error!(
                                "Updating fleet with no missiles",
                                "How did this happen?"
                            ))?;
                            let old_missile = missile_types
                                .missile_template
                                .as_mut()
                                .map(|missile_template| {
                                    missile_template
                                        .iter_mut()
                                        .filter(|child| {
                                            MissileTemplateId::from_missile(child)
                                                == MissileTemplateId::from_missile_data(
                                                    &missile_data,
                                                )
                                        })
                                        .next()
                                })
                                .flatten()
                                .unwrap();
                            *old_missile = new_missile.clone();

                            write_fleet(fleet_path, &fleet)?;

                            debug!("Successfully updated '{}'", fleet_path.display());
                        }

                        info!(
                            "Successfully updated {} fleets with new missile: '{}'",
                            fleets.len(),
                            missile_data.template_name
                        );

                        Ok(())
                    });
                });
            }

            confirm_dialog
                .show()
                .map_err(|err| my_error!("Failed to open confirmation dialog", err))?;

            Ok(())
        });
    }
}
