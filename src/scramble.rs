use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use rand::Rng;
use schemas::Fleet;
use slint::{Model, VecModel, Weak};
use tracing::{debug, info};

use crate::{
    error::{wrap_errorable_function, Error},
    fleet_io::{read_fleet, write_fleet},
    my_error, FleetData, MainWindow,
};

pub fn scramble_fleet_file(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();

    info!("Scrambling fleet '{}'", path.display());

    let mut fleet = read_fleet(path)?;

    scramble_fleet(&mut fleet)?;

    let output_path = path
        .parent()
        .map(|parent| parent.join(&fleet.name).with_extension("fleet"))
        .unwrap_or(PathBuf::from(&fleet.name).with_extension("fleet"));
    info!("Outputing scrambled fleet to '{}'", output_path.display());
    write_fleet(&output_path, &fleet)?;

    Ok(())
}

fn scramble_fleet(fleet: &mut Fleet) -> Result<(), Error> {
    let mut rng = rand::rng();

    // Scramble missile names
    let mut missile_dict = HashMap::new();
    'block: {
        let Some(missile_types) = &mut fleet.missile_types else {
            break 'block;
        };
        let Some(missile_templates) = &mut missile_types.missile_template else {
            break 'block;
        };
        for missile_template in missile_templates {
            let new_name = format!("{:X}", rng.random::<u32>());
            debug!("Scrambling missile '{}' to '{}'", &missile_template.nickname, &new_name);
            missile_dict.insert(
                format!(
                    "$MODMIS$/{} {}",
                    missile_template.designation, missile_template.nickname
                ),
                format!("$MODMIS$/{} {}", missile_template.designation, new_name),
            );
            missile_template.nickname = new_name;
        }
    }

    let mut craft_dict = HashMap::new();
    'block: {
        let Some(craft_types) = &mut fleet.craft_types else {
            break 'block;
        };
        let Some(craft_templates) = &mut craft_types.craft_template else {
            break 'block;
        };
        for craft_template in craft_templates {
            let new_name = format!("{:X}", rng.random::<u32>());
            debug!(
                "Scrambling craft template '{}' to '{}'",
                &craft_template.nickname, &new_name
            );

            let designation = frame_key_to_designation(craft_template.frame_key.as_str());
            let designation_suffix = craft_template.designation_suffix.clone().unwrap_or_default();
            craft_dict.insert(
                format!(
                    "$CRAFT$/{}{} {}",
                    designation, designation_suffix, craft_template.nickname
                ),
                format!("$CRAFT$/{}{} {}", designation, designation_suffix, new_name),
            );
            craft_template.nickname = new_name;

            let Some(craft_loadouts) = &mut craft_template.loadouts.craft_loadout else {
                continue;
            };
            for loadout in craft_loadouts {
                let Some(general_loadout_element) = &mut loadout.elements.general_loadout_element else {
                    continue;
                };
                for general_loadout in general_loadout_element {
                    let Some(missile_keys) = (if general_loadout.xsi_type == "MissileSelection" {
                        let Some(missile_keys) = &mut general_loadout.missile_keys else {
                            continue;
                        };
                        Some(&mut missile_keys.string)
                    } else if general_loadout.xsi_type == "VariableSocketLoadout" {
                        let Some(loadout) = &mut general_loadout.loadout else {
                            continue;
                        };
                        if &loadout.xsi_type == "MissileSelection" {
                            loadout
                                .missile_keys
                                .as_mut()
                                .map(|missile_keys| &mut missile_keys.string)
                        } else {
                            None
                        }
                    } else {
                        None
                    }) else {
                        continue;
                    };
                    for missile_key in missile_keys.into_iter() {
                        let Some(text) = &mut missile_key.text else {
                            continue;
                        };
                        let Some(new_key) = missile_dict.get(text) else {
                            continue;
                        };
                        *text = new_key.clone();
                    }
                }
            }
        }
    };

    // Scramble ships
    fleet.name = format!("{}_SCRAMBLED_{:X}", fleet.name.replace(" ", "_"), rng.random::<u32>());
    'block: {
        let Some(ships) = &mut fleet.ships else {
            break 'block;
        };
        let Some(ships) = &mut ships.ship else {
            break 'block;
        };
        for ship in ships {
            // Scramble ship name
            let new_name = format!("{:X}", rng.random::<u32>());
            debug!("Scrambling ship '{}' to '{}'", &ship.name, &new_name);
            if ship.callsign.is_none() || ship.callsign == Some("".to_string()) {
                ship.callsign = Some(ship.name.clone());
            }
            ship.name = new_name;

            // Apply new missile names to missiles in compartments or mounts
            for hull_socket in ship.socket_map.hull_socket.iter_mut() {
                let Some(component_data) = &mut hull_socket.component_data else {
                    continue;
                };

                // Check for VLS/CLS/TLS
                if component_data.xsi_type.as_str() == "CellLauncherData"
                    || component_data.xsi_type.as_str() == "ResizableCellLauncherData"
                {
                    let Some(missile_load) = &mut component_data.missile_load else {
                        continue;
                    };
                    let Some(mag_save_datas) = &mut missile_load.mag_save_data else {
                        continue;
                    };
                    for mag_save_data in mag_save_datas.iter_mut() {
                        if let Some(new_key) = missile_dict.get(&mag_save_data.munition_key) {
                            mag_save_data.munition_key = new_key.clone();
                        };
                    }
                }
                // Check for magazines
                else if component_data.xsi_type.as_str() == "BulkMagazineData" {
                    let Some(load) = &mut component_data.load else { continue };
                    let Some(mag_save_datas) = &mut load.mag_save_data else {
                        continue;
                    };
                    for mag_save_data in mag_save_datas.iter_mut() {
                        if let Some(new_key) = missile_dict.get(&mag_save_data.munition_key) {
                            mag_save_data.munition_key = new_key.clone();
                        };
                    }
                }
                // Check for hangars
                else if component_data.xsi_type.as_str() == "CraftHangarData" {
                    let Some(stored_craft) = &mut component_data.stored_craft else {
                        continue;
                    };
                    let Some(saved_stored_craft) = &mut stored_craft.saved_stored_craft else {
                        continue;
                    };

                    for saved_stored_craft in saved_stored_craft {
                        if let Some(new_key) = craft_dict.get(&saved_stored_craft.craft_template_key) {
                            saved_stored_craft.craft_template_key = new_key.clone();
                        };
                    }
                }
            }
        }
    }

    Ok(())
}

fn frame_key_to_designation(frame_key: &str) -> &'static str {
    match frame_key {
        // ANS Frames
        "Stock/AN Skiff" => "RS-35",
        "Stock/AN Interceptor" => "PF-440",
        "Stock/AN SEWAC" => "E-44",
        "Stock/AN Bomber" => "B-86",
        // OSPN Frames
        "Stock/OSP Skiff" => "RS-35",
        "Stock/OSP Interceptor" => "PF-386",
        "Stock/OSP Scout" => "RF-286",
        "Stock/OSP Bomber" => "B-45",
        a => panic!("Unknown craft type: {}", a),
    }
}

pub fn on_scramble_fleet_handler(main_window: Weak<MainWindow>, fleets_model: Rc<VecModel<FleetData>>) -> impl Fn() {
    move || {
        let main_window = main_window.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            let cur_fleet_idx = main_window.get_cur_fleet_idx();

            let fleet_data = fleets_model
                .iter()
                .nth(cur_fleet_idx as usize)
                .ok_or(my_error!("Selected fleet doesn't exist", "Index out of bounds"))?;

            scramble_fleet_file(&fleet_data.path)?;

            Ok(())
        });
    }
}
