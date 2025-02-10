use std::rc::Rc;

use slint::{Model, ModelRc, ToSharedString, VecModel, Weak};
use tracing::{debug, trace};

use super::{BULKER_SEGMENTS, CONTAINER_BOWS, CONTAINER_CORES, CONTAINER_STERNS};
use crate::{
    error::{wrap_errorable_function, wrap_errorable_function_fe},
    fleet_editor::{BRIDGE_MODELS, BULK_BOWS, BULK_CORES, BULK_STERNS},
    fleet_io::{read_fleet, write_fleet},
    my_error, DressingSelections, DressingSlot, DressingSlots, FleetData, FleetEditorWindow,
    LinerHullConfig, MainWindow,
};

pub fn on_load_dressings_handler(
    main_window_weak: Weak<MainWindow>,
    window_weak: Weak<FleetEditorWindow>,
) -> impl Fn(LinerHullConfig) {
    move |hull_config| {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            let window = window_weak.unwrap();

            let mut dressing_slots = DressingSlots::default();

            debug!(?hull_config, "Determining dressing slots for");

            trace!("Loading bow dressings");
            dressing_slots.bow = match hull_config.segment_bow {
                0 => ModelRc::from([
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Top crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Bottom crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Flat arm bottom".to_shared_string(),
                        ]),
                    },
                ]),
                1 => ModelRc::from([
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Tanks".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Top crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Flat arm top".to_shared_string(),
                        ]),
                    },
                ]),
                2 => ModelRc::from([
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Flat arm top".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Triple arm bottom".to_shared_string(),
                            "Double arm bottom".to_shared_string(),
                        ]),
                    },
                ]),
                _ => panic!(),
            };
            trace!("Loading core dressings");
            dressing_slots.core = match hull_config.segment_core {
                0 => ModelRc::from([
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Top crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Bottom crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Vertical arm top".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Flat arm bottom".to_shared_string(),
                        ]),
                    },
                ]),
                1 => ModelRc::from([
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Tanks".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Top crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Big tanks".to_shared_string(),
                            "Big crates".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Flat arm top".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Vertical arm top".to_shared_string(),
                        ]),
                    },
                ]),
                2 => ModelRc::from([
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Tanks & crates bottom".to_shared_string(),
                            "Crates bottom".to_shared_string(),
                            "Crates under wings".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Crates above wings".to_shared_string(),
                        ]),
                    },
                    DressingSlot {
                        dressings: ModelRc::from([
                            "None".to_shared_string(),
                            "Vertical arm top".to_shared_string(),
                        ]),
                    },
                ]),
                _ => panic!(),
            };

            trace!("Passing dressing slots to UI");
            window.set_dressing_slots(dressing_slots);

            debug!("Dressing slots loaded");

            Ok(())
        });
    }
}

pub fn on_get_liner_config_handler(
    main_window_weak: Weak<MainWindow>,
    window_weak: Weak<FleetEditorWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn() -> LinerHullConfig {
    move || -> LinerHullConfig {
        let main_window = main_window_weak.unwrap();
        wrap_errorable_function(&main_window, || {
            let window = window_weak.unwrap();
            let cur_idx = main_window.get_cur_fleet_idx();
            let fleet_data = fleets_model.iter().nth(cur_idx as usize).ok_or(my_error!(
                "Selected fleet doesn't exist",
                "cur_fleet_idx points to a nonexistant fleet"
            ))?;

            let mut fleet = read_fleet(&fleet_data.path)?;
            let ship_idx = window.get_selected_ship_idx();
            let ship = fleet
                .ships
                .as_mut()
                .map(|ships| {
                    ships
                        .ship
                        .as_mut()
                        .map(|ships| ships.get_mut(ship_idx as usize))
                })
                .flatten()
                .flatten()
                .ok_or(my_error!(
                    "The selected ship idx doesn't exist",
                    "This is a bug"
                ))?;

            debug!(
                "Reading liner config for '{}' in '{}'",
                &ship.name, &fleet.name
            );

            let bow_key_list;
            let core_key_list;
            let stern_key_list;

            if ship.hull_type.as_str() == "Stock/Bulk Hauler" {
                trace!("Selected ship is a Marauder. Loading segment lists");
                bow_key_list = BULK_BOWS.iter();
                core_key_list = BULK_CORES.iter();
                stern_key_list = BULK_STERNS.iter();
            } else {
                trace!("Selected ship is a Moorline. Loading segment lists");
                bow_key_list = CONTAINER_BOWS.iter();
                core_key_list = CONTAINER_CORES.iter();
                stern_key_list = CONTAINER_STERNS.iter();
            }

            let hull_config = ship
                .hull_config
                .as_mut()
                .expect("expected liner to have a HullConfig");
            debug!("Reading segment configurations");
            let mut segment_configurations =
                hull_config.primary_structure.segment_configuration.iter();

            // Bow
            let segment_bow = segment_configurations.next().unwrap();
            let segment_bow_model_idx = bow_key_list
                .take_while(|skey| **skey != segment_bow.key.as_str())
                .count() as i32;
            let segment_bow_dressing = segment_bow
                .dressing
                .int
                .as_ref()
                .map(|x| {
                    x.iter()
                        .map(|child| child.parse::<i32>().unwrap() + 1)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            trace!(dressings = ?segment_bow_dressing, "Loaded bow dressing");

            // Core
            let segment_core = segment_configurations.next().unwrap();
            let segment_core_model_idx = core_key_list
                .take_while(|skey| **skey != segment_core.key.as_str())
                .count() as i32;
            let segment_core_dressing = segment_core
                .dressing
                .int
                .as_ref()
                .map(|x| {
                    x.iter()
                        .map(|child| child.parse::<i32>().unwrap() + 1)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            trace!(dressings = ?segment_core_dressing, "Loaded core dressing");

            // Stern
            let segment_stern = segment_configurations.next().unwrap();
            let segment_stern_model_idx = stern_key_list
                .take_while(|skey| **skey != segment_stern.key.as_str())
                .count() as i32;

            debug!("Reading superstructure configuration");
            let secondary_structure_config =
                &hull_config.secondary_structure.secondary_structure_config;
            let key_idx = BRIDGE_MODELS
                .iter()
                .take_while(|skey| skey != &&&secondary_structure_config.key)
                .count();
            let bridge_segment = secondary_structure_config.segment.parse::<i32>().unwrap();
            let bridge_snappoint = secondary_structure_config
                .snap_point
                .parse::<i32>()
                .unwrap();

            let liner_hull_config = LinerHullConfig {
                bridge_model: key_idx as i32,
                bridge_segment,
                bridge_snappoint,
                segment_bow: segment_bow_model_idx,
                segment_core: segment_core_model_idx,
                segment_stern: segment_stern_model_idx,
                dressings: DressingSelections {
                    bow: segment_bow_dressing.as_slice().into(),
                    core: segment_core_dressing.as_slice().into(),
                },
            };

            debug!(hull_config = ?liner_hull_config, "Loaded liner hull config");

            Ok(liner_hull_config)
        })
        .unwrap_or_default()
    }
}

pub fn on_save_liner_config_handler(
    main_window_weak: Weak<MainWindow>,
    window_weak: Weak<FleetEditorWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn(LinerHullConfig) {
    move |hull_config| {
        let main_window = main_window_weak.unwrap();
        let window = window_weak.unwrap();
        let LinerHullConfig {
            segment_bow,
            segment_core,
            segment_stern,
            bridge_model,
            bridge_segment,
            bridge_snappoint,
            dressings,
        } = &hull_config;
        let _ = wrap_errorable_function_fe(&window, || {
            let cur_idx = main_window.get_cur_fleet_idx();
            let fleet_data = fleets_model.iter().nth(cur_idx as usize).ok_or(my_error!(
                "Selected fleet doesn't exist",
                "cur_fleet_idx points to a nonexistant fleet"
            ))?;
            let mut fleet = read_fleet(&fleet_data.path)?;
            let ship_idx = window.get_selected_ship_idx();
            let ship = fleet
                .ships
                .as_mut()
                .map(|ships| {
                    ships
                        .ship
                        .as_mut()
                        .map(|ships| ships.get_mut(ship_idx as usize))
                })
                .flatten()
                .flatten()
                .ok_or(my_error!(
                    "The selected ship idx doesn't exist",
                    "This is a bug"
                ))?;

            debug!(
                "Saving liner config for '{}' in '{}'",
                ship.name, &fleet_data.name
            );

            let liner_type = match ship.hull_type.as_str() {
                "Stock/Bulk Hauler" => "Bulk",
                "Stock/Container Hauler" => "Container",
                "Stock/Container Hauler Refit" => "Container",
                hull_type => {
                    // This should never occur unless the UI is broken.
                    return Err(my_error!(
                        "Invalid HullType",
                        format!("'{}' is not a line ship", hull_type)
                    ));
                }
            };
            if let Some(err) = match liner_type {
                "Bulk" => is_marauder_config_allowed(&hull_config),
                "Container" => is_moorline_config_allowed(&hull_config),
                // Already checked
                _ => panic!(),
            } {
                return Err(my_error!("Liner hull config not allowed", err));
            }

            trace!("HullType is '{}'", &liner_type);

            let dressing_slots = window.get_dressing_slots();

            let hull_config = ship
                .hull_config
                .as_mut()
                .expect("expected liner to have a HullConfig");
            let primary_structure = &mut hull_config.primary_structure;

            debug!("Editing segment configurations");
            for (idx, child) in primary_structure
                .segment_configuration
                .iter_mut()
                .enumerate()
            {
                trace!("Clearing previous dressings");
                let dressing = match child.dressing.int.as_mut() {
                    Some(dressing) => dressing,
                    None => &mut Vec::new(),
                };
                dressing.clear();

                let segment_type_idx;
                let segment_name;

                match idx {
                    0 => {
                        trace!("Setting bow hull configuration");
                        segment_type_idx = segment_bow;
                        segment_name = "Bow";

                        trace!("Setting bow dressing configuration");
                        for elem in dressings
                            .bow
                            .iter()
                            .map(|i| i - 1)
                            .take(dressing_slots.bow.iter().count())
                        {
                            dressing.push(elem.to_string());
                        }
                    }
                    1 => {
                        trace!("Setting core hull configuration");
                        segment_type_idx = segment_core;
                        segment_name = "Core";

                        trace!("Setting core dressing configuration");
                        for elem in dressings
                            .core
                            .iter()
                            .map(|i| i - 1)
                            .take(dressing_slots.core.iter().count())
                        {
                            dressing.push(elem.to_string());
                        }
                    }
                    2 => {
                        trace!("Setting stern hull configuration");
                        segment_type_idx = segment_stern;
                        segment_name = "Stern";
                    }
                    _ => panic!(),
                };

                let key_lookup_name =
                    format!("{}-{}-{}", liner_type, segment_type_idx, segment_name);
                trace!("Looking up segment key '{}'", &key_lookup_name);
                let key_data = BULKER_SEGMENTS.get(&key_lookup_name.as_str()).unwrap();
                trace!("Returned segment key '{}'", &key_data);

                child.key = key_data.to_string();
            }

            trace!("Setting superstructure configuration");
            let secondary_structure =
                &mut hull_config.secondary_structure.secondary_structure_config;

            let key_lookup_name = format!("Superstructure-{}", bridge_model);
            trace!("Looking up superstructure key '{}'", &key_lookup_name);
            let key_data = BULKER_SEGMENTS.get(&key_lookup_name.as_str()).unwrap();
            trace!("Returned key '{}'", &key_data);
            secondary_structure.key = key_data.to_string();

            secondary_structure.segment = bridge_segment.to_string();
            secondary_structure.snap_point = bridge_snappoint.to_string();

            debug!("Hull configuration complete");

            write_fleet(&fleet_data.path, &fleet)?;

            debug!("Hull configuration saved");

            Ok(())
        });
    }
}

fn is_marauder_config_allowed(hull_config: &LinerHullConfig) -> Option<&'static str> {
    match hull_config.bridge_segment {
        // Bow
        0 => {
            // C
            if hull_config.segment_bow == 2 {
                Some("Bridge cannot be placed on bow section C")
            } else {
                None
            }
        }
        // Core
        1 => {
            // All cores are valid
            None
        }
        // Stern
        2 => {
            // B
            if hull_config.segment_stern == 1 {
                Some("Bridge cannot be placed on stern section B")
            } else {
                None
            }
        }
        _ => Some("Bridge assigned to non-existant section"),
    }
}

fn is_moorline_config_allowed(hull_config: &LinerHullConfig) -> Option<&'static str> {
    match hull_config.bridge_segment {
        // Bow
        0 => {
            // A
            if hull_config.segment_bow == 0 {
                Some("Bridge cannot be placed on bow section A")
            } else {
                None
            }
        }
        // Core
        1 => {
            // All cores are valid
            None
        }
        // Stern
        2 => {
            // All sterns are valid
            None
        }
        _ => Some("Bridge assigned to non-existant section"),
    }
}
