use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    rc::Rc,
};

use slint::{Model, ModelRc, ToSharedString, VecModel, Weak};
use tracing::{debug, trace};
use xml::EmitterConfig;
use xmltree::{AttributeMap, Element, Traversable};

use super::{BULKER_SEGMENTS, CONTAINER_BOWS, CONTAINER_CORES, CONTAINER_STERNS};
use crate::{
    error::wrap_errorable_function,
    fleet_editor::{BRIDGE_MODELS, BULK_BOWS, BULK_CORES, BULK_STERNS},
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

            debug!(
                "Reading liner config for '{}' in '{}'",
                selected_ship_element
                    .get_child("Name")
                    .unwrap()
                    .get_text()
                    .unwrap(),
                &fleet.name
            );

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

            let hull_config = selected_ship_element.get_child("HullConfig").unwrap();
            debug!("Reading segment configurations");
            let primary_structure = hull_config.get_child("PrimaryStructure").unwrap();
            let mut children = primary_structure.get_children().into_iter();

            // Bow
            let segment_bow_elem = children.next().unwrap();
            let segment_bow_key = segment_bow_elem
                .get_child("Key")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string();
            let segment_bow_model_idx = bow_key_list
                .take_while(|skey| **skey != segment_bow_key.as_str())
                .count() as i32;
            let segment_bow_dressing = segment_bow_elem
                .get_child("Dressing")
                .unwrap()
                .get_children()
                .iter()
                .map(|child| {
                    child
                        .get_text()
                        .unwrap()
                        .to_string()
                        .parse::<i32>()
                        .unwrap()
                        + 1
                })
                .collect::<Vec<_>>();
            trace!(dressings = ?segment_bow_dressing, "Loaded bow dressing");

            // Core
            let segment_core_elem = children.next().unwrap();
            let segment_core_key = segment_core_elem
                .get_child("Key")
                .unwrap()
                .get_text()
                .unwrap()
                .to_string();
            let segment_core = core_key_list
                .take_while(|skey| **skey != segment_core_key.as_str())
                .count() as i32;
            let segment_core_dressing = segment_core_elem
                .get_child("Dressing")
                .unwrap()
                .get_children()
                .iter()
                .map(|child| {
                    child
                        .get_text()
                        .unwrap()
                        .to_string()
                        .parse::<i32>()
                        .unwrap()
                        + 1
                })
                .collect::<Vec<_>>();
            trace!(dressings = ?segment_core_dressing, "Loaded core dressing");

            // Stern
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

            debug!("Reading superstructure configuration");
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

            let liner_hull_config = LinerHullConfig {
                bridge_model: key_idx as i32,
                bridge_segment,
                bridge_snappoint,
                segment_bow: segment_bow_model_idx,
                segment_core,
                segment_stern,
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
    move |LinerHullConfig {
              segment_bow,
              segment_core,
              segment_stern,
              bridge_model,
              bridge_segment,
              bridge_snappoint,
              dressings,
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

            debug!(
                "Saving liner config for '{}' in '{}'",
                selected_ship_element
                    .get_child("Name")
                    .unwrap()
                    .get_text()
                    .unwrap(),
                &fleet.name
            );

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
                "Stock/Container Hauler Refit" => "Container",
                _ => {
                    panic!()
                }
            };

            trace!("HullType is '{}'", &liner_type);

            let dressing_slots = window.get_dressing_slots();

            let hull_config = selected_ship_element.get_mut_child("HullConfig").unwrap();
            let primary_structure = hull_config.get_mut_child("PrimaryStructure").unwrap();

            let attr_map = [
                (String::new(), String::new()),
                (
                    String::from("xml"),
                    String::from("http://www.w3.org/XML/1998/namespace"),
                ),
                (
                    String::from("xmlns"),
                    String::from("http://www.w3.org/2000/xmlns/"),
                ),
                (
                    String::from("xsd"),
                    String::from("http://www.w3.org/2001/XMLSchema"),
                ),
                (
                    String::from("xsd"),
                    String::from("http://www.w3.org/2001/XMLSchema-instance"),
                ),
            ];
            let mut namespace = xmltree::Namespace::empty();
            for (prefix, uri) in attr_map {
                namespace.put(prefix, uri);
            }
            // Template to be copied later.
            let int_elem = Element {
                prefix: None,
                namespace: None,
                namespaces: Some(namespace),
                name: String::from("int"),
                attributes: AttributeMap::new(),
                children: vec![],
                attribute_namespaces: HashMap::new(),
            };

            debug!("Editing segment configurations");
            for (idx, child) in primary_structure.children.iter_mut().enumerate() {
                let child = child.as_mut_element().unwrap();

                trace!("Clearing previous dressings");
                let dressing = child.get_mut_child("Dressing").unwrap();
                dressing.children.clear();
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
                            .map(|i| {
                                let mut int_elem = int_elem.clone();
                                int_elem.children = vec![xmltree::XMLNode::Text(i.to_string())];
                                xmltree::XMLNode::Element(int_elem)
                            })
                            .take(dressing_slots.bow.iter().count())
                        {
                            dressing.children.push(elem);
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
                            .map(|i| {
                                let mut int_elem = int_elem.clone();
                                int_elem.children = vec![xmltree::XMLNode::Text(i.to_string())];
                                xmltree::XMLNode::Element(int_elem)
                            })
                            .take(dressing_slots.core.iter().count())
                        {
                            dressing.children.push(elem);
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

                let text_node = xmltree::XMLNode::Text(key_data.to_string());
                let key = child.get_mut_child("Key").unwrap();
                key.children = vec![text_node];
            }

            trace!("Setting superstructure configuration");
            let secondary_structure = hull_config
                .get_mut_child("SecondaryStructure")
                .unwrap()
                .get_mut_child("SecondaryStructureConfig")
                .unwrap();

            let key_lookup_name = format!("Superstructure-{}", bridge_model);
            trace!("Looking up superstructure key '{}'", &key_lookup_name);
            let key_data = BULKER_SEGMENTS.get(&key_lookup_name.as_str()).unwrap();
            trace!("Returned key '{}'", &key_data);
            let key = secondary_structure.get_mut_child("Key").unwrap();
            key.children = vec![xmltree::XMLNode::Text(key_data.to_string())];

            let segment = secondary_structure.get_mut_child("Segment").unwrap();
            segment.children = vec![xmltree::XMLNode::Text(bridge_segment.to_string())];

            let segment = secondary_structure.get_mut_child("SnapPoint").unwrap();
            segment.children = vec![xmltree::XMLNode::Text(bridge_snappoint.to_string())];

            debug!("Hull configuration complete");

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

            debug!("Hull configuration saved");

            Ok(())
        });
    }
}
