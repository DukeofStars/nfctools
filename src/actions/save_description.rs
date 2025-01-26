use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    rc::Rc,
};

use slint::{Model, VecModel};
use tracing::{debug, info, instrument, trace};
use xml::EmitterConfig;
use xmltree::{AttributeMap, Element};

use crate::{error::Error, my_error, FleetData, MainWindow, Tag};

// pub fn on_save_description_handler(
//     main_window_weak: Weak<MainWindow>,
//     fleets_model: Rc<VecModel<FleetData>>,
//     tags_model: Rc<VecModel<Tag>>,
// ) -> impl Fn(SharedString) {
//     move |description| {
//         let main_window = main_window_weak.unwrap();
//         let _ = wrap_errorable_function(&main_window, || {
//             save_fleet_data(
//                 &main_window,
//                 fleets_model.clone(),
//                 tags_model.clone(),
//                 description.to_string(),
//             )
//         });
//     }
// }

#[instrument(skip(main_window, fleets_model, description, tags_model))]
pub fn save_fleet_data(
    main_window: &MainWindow,
    fleets_model: Rc<VecModel<FleetData>>,
    tags_model: Rc<VecModel<Tag>>,
    description: String,
) -> Result<(), Error> {
    let cur_fleet_idx = main_window.get_cur_fleet_idx();
    info!("Saving description for fleet {}", cur_fleet_idx);
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

    debug!("Inserting tags into description");
    let description = format!(
        "Tags: {}\n{}",
        tags_model
            .iter()
            .map(|tag| format!(
                "<color=#{:02x}{:02x}{:02x}>{}</color>",
                tag.color.red(),
                tag.color.green(),
                tag.color.blue(),
                tag.name
            ))
            .collect::<Vec<_>>()
            .join(" "),
        description,
    );

    let mut element = {
        debug!("Opening fleet file");
        let fleet_file = File::open(&fleet.path).map_err(|err| {
            my_error!(
                format!("Failed to open fleet '{}'", fleet.path.to_string()),
                err
            )
        })?;
        trace!("Parsing fleet file");
        Element::parse(fleet_file).map_err(|err| my_error!("Failed to parse fleet file", err))?
    };

    let text_node = xmltree::XMLNode::Text((&description).to_string());

    if description.is_empty() {
        debug!("Not inserting new element, description is empty");
        // It doesn't actually affect the data, but personally I dislike the idea of leaving an empty Description element lying around.
        let _ = element.take_child("Description");
    } else if let Some(description_elem) = element.get_mut_child("Description") {
        debug!("Overwriting old description");
        description_elem.children = vec![text_node];
    } else {
        debug!("Inserting new description");
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
        // Put the description at the start so it is easier to find and read.
        let mut new_children = vec![xmltree::XMLNode::Element(description_elem)];
        new_children.append(&mut element.children);
        element.children = new_children;
    }

    {
        debug!("Deleting previous fleet file");
        std::fs::remove_file(&fleet.path)
            .map_err(|err| my_error!("Failed to delete previous fleet file", err))?;
        debug!("Opening new fleet file");
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
        debug!("Writing to fleet file");
        element
            .write_with_config(fleet_file, config)
            .map_err(|err| {
                my_error!(
                    format!("Failed to write to fleet file '{}'", fleet.path.to_string()),
                    err
                )
            })?;
    }

    info!("Fleet description saved successfully");

    Ok(())
}
