use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{Debug, Display},
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use fleet_info_reader::FleetInfoReader;
use merge::{Reader, Writer};
use slint::{ComponentHandle, Model, ToSharedString};
use tracing::{debug, error, info, level_filters::LevelFilter, trace};
use xml::{reader::EventReader, EmitterConfig};
use xmltree::{AttributeMap, Element};

mod fleet_info_reader;
mod merge;

slint::include_modules!();

const FLEETS_ROOT_DIR: &str =
    r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets\"#;

struct Error {
    title: String,
    error: Box<dyn Display>,
}
impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("title", &self.title)
            .field("error", &self.error.to_string())
            .finish()
    }
}

macro_rules! my_error {
    ($title: expr, $error: expr) => {
        Error {
            title: $title.to_string(),
            error: Box::new($error),
        }
    };
    ($title: expr, $error: expr,) => {
        Error {
            title: $title.to_string(),
            error: Box::new($error),
        }
    };
}

fn wrap_errorable_function(
    main_window: &MainWindow,
    mut f: impl FnMut() -> Result<(), Error>,
) -> Option<Error> {
    if let Err(err) = f() {
        error!(%err.error, "{}", err.title);
        main_window.invoke_show_error_popup((&err.title).into(), (&err.error).to_string().into());
        return Some(err);
    }
    None
}

fn load_fleets(path: impl AsRef<Path>) -> color_eyre::Result<Vec<FleetData>> {
    debug!("Loading fleets from {}", path.as_ref().display());
    let mut output = vec![];
    load_fleets_rec(path, &mut output)?;

    debug!("Loaded {} fleets", output.len());

    Ok(output)
}
fn load_fleets_rec(path: impl AsRef<Path>, output: &mut Vec<FleetData>) -> color_eyre::Result<()> {
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
            load_fleets_rec(child.path(), output)?;
        }
        if file_type.is_file() {
            if child.path().extension().map(|s| s.to_str()) != Some(Some("fleet".into())) {
                continue;
            }
            let fleet_info_reader = FleetInfoReader::new(File::open(child.path())?);
            let fleet_name = fleet_info_reader.get_value("Fleet/Name");
            let path = child.path().to_path_buf();
            let short_path = path
                .strip_prefix(FLEETS_ROOT_DIR)?
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

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();
    color_eyre::install()?;

    info!("Starting NebTools");

    let main_window = MainWindow::new()?;

    let fleets = load_fleets(FLEETS_ROOT_DIR)?;

    let fleets_model = std::rc::Rc::new(slint::VecModel::from(fleets));
    main_window.set_fleets(fleets_model.clone().into());
    debug!("Fleets passed to UI");

    debug!("Setting up callbacks");

    {
        let fleets_model = fleets_model.clone();
        let main_window_weak = main_window.as_weak();
        main_window.on_open_fleet_editor(move || {
            let main_window = main_window_weak.unwrap();
            wrap_errorable_function(&main_window, || {
                let window = FleetEditorWindow::new()
                    .map_err(|err| my_error!("Failed to create fleet editor window", err))
                    .unwrap();

                let cur_idx = main_window.get_cur_fleet_idx();
                if cur_idx == -1 {
                    return Err(my_error!("No fleet selected", ""));
                }
                let fleet = fleets_model.iter().nth(cur_idx as usize).ok_or(my_error!(
                    "Selected fleet doesn't exist",
                    "cur_fleet_idx points to a nonexistant fleet"
                ))?;

                window.set_fleet_name(fleet.name);

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
                let ships_elem = element
                    .get_child("Ships")
                    .ok_or(my_error!("Failed to get ships list", "Fleet has no ships"))?;

                let ships = ships_elem
                    .children
                    .iter()
                    .map(|ship_elem| {
                        let ship_elem = ship_elem
                            .as_element()
                            .ok_or(my_error!("Invalid fleet file", "Ship is not an element"))?;

                        let name = ship_elem
                            .get_child("Name")
                            .ok_or(my_error!("Invalid fleet file", "Ship has no name"))?
                            .get_text()
                            .ok_or(my_error!("Invalid fleet file", "Ship has no name"))?
                            .to_shared_string();
                        let hulltype = ship_elem
                            .get_child("HullType")
                            .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                            .get_text()
                            .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                            .to_shared_string();
                        let cost: i32 = ship_elem
                            .get_child("Cost")
                            .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                            .get_text()
                            .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                            .parse()
                            .map_err(|err| {
                                my_error!(
                                    "Invalid fleet file",
                                    format!("Failed to parse cost: {}", err)
                                )
                            })?;

                        Ok(ShipData {
                            class: hulltype,
                            name,
                            cost,
                        })
                    })
                    .collect::<Result<Vec<ShipData>, Error>>()?;

                let ships_model = std::rc::Rc::new(slint::VecModel::from(ships));
                window.set_ships(ships_model.clone().into());

                window
                    .show()
                    .map_err(|err| my_error!("Could not show fleet editor window.", err))
                    .unwrap();

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_reload_fleets(move || {
            let main_window = main_window_weak.unwrap();
            wrap_errorable_function(&main_window, || {
                debug!("Reloading fleets list");
                let fleets_path =
                    r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets\"#;
                let fleets = load_fleets(fleets_path)
                    .map_err(|err| my_error!("Failed to load fleets", err))?;
                fleets_model.set_vec(fleets);

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_viewing(move |idx| {
            let main_window = main_window_weak.unwrap();
            wrap_errorable_function(&main_window_weak.unwrap(), || {
                if !main_window.get_multi_selecting() {
                    fleets_model.set_vec(
                        fleets_model
                            .iter()
                            .enumerate()
                            .map(|(f_idx, mut fleet)| {
                                if f_idx as i32 != idx {
                                    fleet.selected = false;
                                }
                                fleet
                            })
                            .collect::<Vec<_>>(),
                    );
                }

                // idx is set to -1 when a fleet is unselected, meaning no fleet is selected.
                if idx == -1 {
                    return Ok(());
                }

                let fleet = fleets_model.iter().nth(idx as usize).ok_or(my_error!(
                    "Selected fleet doesn't exist",
                    "cur_fleet_idx points to a nonexistant fleet"
                ))?;
                trace!("Viewing fleet {}: '{}'", idx, fleet.name);
                let fleet_info_reader =
                    FleetInfoReader::new(File::open(fleet.path.to_string()).map_err(|err| {
                        my_error!(
                            format!("Failed to open fleet '{}'", fleet.path.to_string()),
                            err
                        )
                    })?);
                let description = fleet_info_reader.get_value("Fleet/Description");
                trace!(%description, "Found description");

                main_window.invoke_update_description(description.into());

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_save_description(move |description| {
            let main_window = main_window_weak.unwrap();

            wrap_errorable_function(&main_window, || {
                let cur_fleet_idx = main_window.get_cur_fleet_idx();
                if cur_fleet_idx == -1 {
                    return Err(my_error!("No fleet selected", ""));
                }
                let fleet = fleets_model
                    .iter()
                    .nth(cur_fleet_idx as usize)
                    .ok_or(my_error!(
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

                let text_node = xmltree::XMLNode::Text((&description).to_string());

                if description.is_empty() {
                    trace!("Not inserting new element, description is empty");
                    // It doesn't actually affect the data, but personally I dislike the idea of leaving an empty Description element lying around.
                    let _ = element.take_child("Description");
                } else if let Some(description_elem) = element.get_mut_child("Description") {
                    trace!("Overwriting old description");
                    description_elem.children = vec![text_node];
                } else {
                    trace!("Inserting new description");
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
                    let description_elem = Element {
                        prefix: None,
                        namespace: None,
                        namespaces: Some(namespace),
                        name: String::from("Description"),
                        attributes: AttributeMap::new(),
                        children: vec![text_node],
                        attribute_namespaces: HashMap::new(),
                    };
                    // For some reason the new element must be at the start of the list otherwise the fleet file is corrupted. ¯\_(ツ)_/¯
                    let mut new_children = vec![xmltree::XMLNode::Element(description_elem)];
                    new_children.append(&mut element.children);
                    element.children = new_children;
                }

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
                                format!(
                                    "Failed to write to fleet file '{}'",
                                    fleet.path.to_string()
                                ),
                                err
                            )
                        })?;
                }

                debug!("Fleet description saved");

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_merge(move |merge_output_name| {
            let main_window = main_window_weak.unwrap();
            wrap_errorable_function(&main_window, || {
                let merge_output_name = merge_output_name.to_string().trim().to_string();
                debug!("Merging fleets into '{}'", merge_output_name);
                if merge_output_name == "" {
                    return Err(my_error!(
                        "No merge output name",
                        "You must set an output name for the merged fleets"
                    ));
                }

                let selected_fleets = fleets_model
                    .iter()
                    .filter(|f| f.selected)
                    .collect::<Vec<_>>();
                debug!(
                    "Merging fleets {:?}",
                    selected_fleets.iter().map(|f| &f.name).collect::<Vec<_>>()
                );
                let first_fleet = &selected_fleets[0];
                trace!("Primary fleet is '{}'", first_fleet.name);

                let mut ships = Vec::new();
                let mut missiles = Vec::new();
                if !selected_fleets
                    .iter()
                    // Skip the primary fleet as it's ships are included by default
                    .skip(1)
                    .filter_map(|fleet| {
                        let file = File::open(fleet.path.to_string());
                        if file.is_err() {
                            return Some(());
                        }
                        let file = file.unwrap();

                        trace!("Pulling ships from fleet at '{}'", fleet.path);
                        Reader::new(EventReader::new(&file), &mut ships, "Ships", "Ship")
                            .run_until_complete();

                        let file = File::open(fleet.path.to_string());
                        if file.is_err() {
                            return Some(());
                        }
                        let file = file.unwrap();

                        trace!("Pulling missile types from fleet at '{}'", fleet.path);
                        Reader::new(
                            EventReader::new(&file),
                            &mut missiles,
                            "MissileTypes",
                            "MissileTemplate",
                        )
                        .run_until_complete();

                        None
                    })
                    .collect::<Vec<_>>()
                    .is_empty()
                {
                    // An error occurred in one of the fleets.
                    return Err(my_error!(
                        "Failed to merge fleets",
                        "One or more fleets could not be parsed"
                    ));
                };

                let output_path = PathBuf::from(
                    r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets\"#,
                )
                .join(&merge_output_name)
                .with_extension("fleet");
                let mut output = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&output_path)
                    .map_err(|err| {
                        my_error!(
                            format!("Failed to write to file '{}'", output_path.display()),
                            err
                        )
                    })?;
                let mut writer = Writer::new(
                    &mut output,
                    EventReader::new(File::open(first_fleet.path.to_string()).map_err(|err| {
                        my_error!(
                            format!("Failed to open fleet {}", first_fleet.path.to_string()),
                            err
                        )
                    })?),
                    ships,
                    missiles,
                    merge_output_name,
                );
                writer.run_until_complete();
                debug!("Merge complete");

                // A new fleet has been created, so reload the fleet list.
                main_window.invoke_reload_fleets();

                Ok(())
            });
        });
    }

    main_window.run()?;

    Ok(())
}
