use std::{
    fs::{File, OpenOptions},
    rc::Rc,
};

use slint::{Model, VecModel, Weak};
use tracing::trace;
use xml::EmitterConfig;
use xmltree::{Element, Traversable};

use super::{BULKER_SEGMENTS, CONTAINER_BOWS, CONTAINER_CORES, CONTAINER_STERNS};
use crate::{
    error::wrap_errorable_function,
    fleet_editor::{BRIDGE_MODELS, BULK_BOWS, BULK_CORES, BULK_STERNS},
    my_error, FleetData, FleetEditorWindow, LinerHullConfig, MainWindow,
};

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
            let fleet = fleets_model.iter().nth(cur_idx as usize).ok_or(my_error!(
                "Selected fleet doesn't exist",
                "cur_fleet_idx points to a nonexistant fleet"
            ))?;
            let element = {
                trace!("Opening fleet file");
                let fleet_file = File::open(&fleet.path).map_err(|err| {
                    my_error!(
                        format!("Failed to open fleet '{}'", fleet.path.to_string()),
                        err
                    )
                })?;
                trace!("Parsing fleet file");
                Element::parse(fleet_file)
                    .map_err(|err| my_error!("Failed to parse fleet file", err))?
            };
            let ship_idx = window.get_selected_ship_idx();
            let selected_ship_element = element
                .get_child("Ships")
                .unwrap()
                .children
                .get(ship_idx as usize)
                .unwrap()
                .as_element()
                .unwrap();

            let bow_key_list;
            let core_key_list;
            let stern_key_list;

            if selected_ship_element
                .get_child("HullType")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string()
                .as_str()
                == "Stock/Bulk Hauler"
            {
                bow_key_list = BULK_BOWS.iter();
                core_key_list = BULK_CORES.iter();
                stern_key_list = BULK_STERNS.iter();
            } else {
                bow_key_list = CONTAINER_BOWS.iter();
                core_key_list = CONTAINER_CORES.iter();
                stern_key_list = CONTAINER_STERNS.iter();
            }

            let hull_config = selected_ship_element.get_child("HullConfig").unwrap();
            let primary_structure = hull_config.get_child("PrimaryStructure").unwrap();
            let mut children = primary_structure.get_children().into_iter();
            let segment_bow = children
                .next()
                .unwrap()
                .get_child("Key")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string();
            let segment_bow = bow_key_list
                .take_while(|skey| **skey != segment_bow.as_str())
                .count() as i32;
            let segment_core = children
                .next()
                .unwrap()
                .get_child("Key")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string();
            let segment_core = core_key_list
                .take_while(|skey| **skey != segment_core.as_str())
                .count() as i32;
            let segment_stern = children
                .next()
                .unwrap()
                .get_child("Key")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string();
            let segment_stern = stern_key_list
                .take_while(|skey| **skey != segment_stern.as_str())
                .count() as i32;

            let secondary_structure = hull_config.get_child("SecondaryStructure").unwrap();
            let secondary_structure_config = secondary_structure
                .get_child("SecondaryStructureConfig")
                .unwrap();
            let key = secondary_structure_config
                .get_child("Key")
                .unwrap()
                .get_text()
                .unwrap();
            let key_idx = BRIDGE_MODELS
                .iter()
                .take_while(|skey| skey.to_string() != key.to_string())
                .count();
            let bridge_segment = secondary_structure_config
                .get_child("Segment")
                .unwrap()
                .get_text()
                .unwrap()
                .parse::<i32>()
                .unwrap();
            let bridge_snappoint = secondary_structure_config
                .get_child("SnapPoint")
                .unwrap()
                .get_text()
                .unwrap()
                .parse::<i32>()
                .unwrap();

            Ok(LinerHullConfig {
                bridge_model: key_idx as i32,
                bridge_segment,
                bridge_snappoint,
                segment_bow,
                segment_core,
                segment_stern,
            })
        })
        .unwrap_or_default()
    }
}

pub fn on_save_liner_config_handler(
    main_window_weak: Weak<MainWindow>,
    window_weak: Weak<FleetEditorWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn(LinerHullConfig) {
    move |LinerHullConfig {
              segment_bow,
              segment_core,
              segment_stern,
              bridge_model,
              bridge_segment,
              bridge_snappoint,
          }| {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            let window = window_weak.unwrap();
            let cur_idx = main_window.get_cur_fleet_idx();
            let fleet = fleets_model.iter().nth(cur_idx as usize).ok_or(my_error!(
                "Selected fleet doesn't exist",
                "cur_fleet_idx points to a nonexistant fleet"
            ))?;
            let mut element = {
                trace!("Opening fleet file");
                let fleet_file = File::open(&fleet.path).map_err(|err| {
                    my_error!(
                        format!("Failed to open fleet '{}'", fleet.path.to_string()),
                        err
                    )
                })?;
                trace!("Parsing fleet file");
                Element::parse(fleet_file)
                    .map_err(|err| my_error!("Failed to parse fleet file", err))?
            };
            let ship_idx = window.get_selected_ship_idx();
            let selected_ship_element = element
                .get_mut_child("Ships")
                .unwrap()
                .children
                .get_mut(ship_idx as usize)
                .unwrap()
                .as_mut_element()
                .unwrap();

            let liner_type = match selected_ship_element
                .get_child("HullType")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string()
                .as_str()
            {
                "Stock/Bulk Hauler" => "Bulk",
                "Stock/Container Hauler" => "Container",
                _ => {
                    panic!()
                }
            };

            let hull_config = selected_ship_element.get_mut_child("HullConfig").unwrap();
            let primary_structure = hull_config.get_mut_child("PrimaryStructure").unwrap();
            for (idx, child) in primary_structure.children.iter_mut().enumerate() {
                let child = child.as_mut_element().unwrap();
                let key = child.get_mut_child("Key").unwrap();

                let segment_type_idx;
                let segment_name = match idx {
                    0 => {
                        segment_type_idx = segment_bow;
                        "Bow"
                    }
                    1 => {
                        segment_type_idx = segment_core;
                        "Core"
                    }
                    2 => {
                        segment_type_idx = segment_stern;
                        "Stern"
                    }
                    _ => panic!(),
                };

                let key_lookup_name =
                    format!("{}-{}-{}", liner_type, segment_type_idx, segment_name);
                let key_data = BULKER_SEGMENTS.get(&key_lookup_name.as_str()).unwrap();

                let text_node = xmltree::XMLNode::Text(key_data.to_string());
                key.children = vec![text_node];
            }

            let secondary_structure = hull_config
                .get_mut_child("SecondaryStructure")
                .unwrap()
                .get_mut_child("SecondaryStructureConfig")
                .unwrap();

            let key_lookup_name = format!("Superstructure-{}", bridge_model);
            let key_data = BULKER_SEGMENTS.get(&key_lookup_name.as_str()).unwrap();
            let key = secondary_structure.get_mut_child("Key").unwrap();
            key.children = vec![xmltree::XMLNode::Text(key_data.to_string())];

            let segment = secondary_structure.get_mut_child("Segment").unwrap();
            segment.children = vec![xmltree::XMLNode::Text(bridge_segment.to_string())];

            let segment = secondary_structure.get_mut_child("SnapPoint").unwrap();
            segment.children = vec![xmltree::XMLNode::Text(bridge_snappoint.to_string())];

            {
                std::fs::remove_file(&fleet.path)
                    .map_err(|err| my_error!("Failed to delete previous fleet file", err))?;
                trace!("Saving file");
                let fleet_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&fleet.path)
                    .map_err(|err| {
                        my_error!(
                            format!("Failed to open fleet '{}'", fleet.path.to_string()),
                            err
                        )
                    })?;
                let config = EmitterConfig::new().perform_indent(true);
                element
                    .write_with_config(fleet_file, config)
                    .map_err(|err| {
                        my_error!(
                            format!("Failed to write to fleet file '{}'", fleet.path.to_string()),
                            err
                        )
                    })?;
            }

            Ok(())
        });
    }
}
