use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    rc::Rc,
};

use slint::{Model, SharedString, VecModel, Weak};
use tracing::{debug, trace};
use xml::EmitterConfig;
use xmltree::{AttributeMap, Element};

use crate::{error::wrap_errorable_function, my_error, FleetData, MainWindow};

pub fn on_save_description_handler(
    main_window_weak: Weak<MainWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn(SharedString) {
    move |description| {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
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
                // Put the description at the start so it is easier to find and read.
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
                            format!("Failed to write to fleet file '{}'", fleet.path.to_string()),
                            err
                        )
                    })?;
            }

            debug!("Fleet description saved");

            Ok(())
        });
    }
}
