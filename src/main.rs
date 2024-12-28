use std::{
    cmp::Ordering,
    fmt::Display,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use fleet_info_reader::FleetInfoReader;
use merge::{Reader, Writer};
use slint::Model;
use tracing::{debug, error, info, level_filters::LevelFilter, trace};
use xml::reader::EventReader;
use xmltree::Element;

mod fleet_info_reader;
mod merge;

slint::include_modules!();

struct Error {
    title: String,
    error: Box<dyn Display>,
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
            let fleet_data = FleetData {
                path: child.path().to_path_buf().to_str().unwrap().into(),
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

    let fleets_path = r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets\"#;
    let fleets = load_fleets(fleets_path)?;

    let fleets_model = std::rc::Rc::new(slint::VecModel::from(fleets));
    main_window.set_fleets(fleets_model.clone().into());
    debug!("Fleets passed to UI");

    debug!("Setting up callbacks");
    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_viewing(move |idx| {
            let main_window = main_window_weak.unwrap();
            wrap_errorable_function(&main_window_weak.unwrap(), || {
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

                main_window.set_cur_fleet_description(description.into());
                main_window.invoke_update_description();

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_save_description(move || {
            let main_window = main_window_weak.unwrap();

            wrap_errorable_function(&main_window, || {
                let cur_description = main_window.get_cur_fleet_description();

                let cur_fleet_idx = main_window.get_cur_fleet_idx();
                let fleet = fleets_model
                    .iter()
                    .nth(cur_fleet_idx as usize)
                    .ok_or(my_error!(
                        "Selected fleet doesn't exist",
                        "cur_fleet_idx points to a nonexistant fleet"
                    ))?;

                trace!("Opening fleet file");
                let fleet_file = File::open(&fleet.path).map_err(|err| {
                    my_error!(
                        format!("Failed to open fleet '{}'", fleet.path.to_string()),
                        err
                    )
                })?;

                trace!("Parsing fleet file");
                let mut element = Element::parse(fleet_file)
                    .map_err(|err| my_error!("Failed to parse fleet file", err))?;

                _ = element.take_child("Description");

                trace!("Inserting new description");
                let mut description_elem = Element::new("Description");
                description_elem
                    .children
                    .push(xmltree::XMLNode::Text((&cur_description).to_string()));

                // For some reason the new element must be at the start of the list otherwise the fleet file is corrupted. ¯\_(ツ)_/¯
                let mut new_children = vec![xmltree::XMLNode::Element(description_elem)];
                new_children.append(&mut element.children);
                element.children = new_children;

                trace!("Saving file");
                let fleet_file =
                    OpenOptions::new()
                        .write(true)
                        .open(&fleet.path)
                        .map_err(|err| {
                            my_error!(
                                format!("Failed to open fleet '{}'", fleet.path.to_string()),
                                err
                            )
                        })?;
                element.write(fleet_file).map_err(|err| {
                    my_error!(
                        format!("Failed to write to fleet file '{}'", fleet.path.to_string()),
                        err
                    )
                })?;

                debug!("Fleet description saved");

                Ok(())
            });
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_merge(move || {
            let main_window = main_window_weak.unwrap();
            wrap_errorable_function(&main_window, || {
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

                let merge_output_name = main_window
                    .get_merge_output_name()
                    .to_string()
                    .trim()
                    .to_string();
                debug!("Merging fleets into '{}'", merge_output_name);
                if merge_output_name == "" {
                    return Err(my_error!(
                        "No merge output name",
                        "You must set an output name for the merged fleets"
                    ));
                }

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

                Ok(())
            });
        });
    }

    main_window.run()?;

    Ok(())
}
