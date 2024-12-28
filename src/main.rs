use slint::Model;
use std::{
    cmp::Ordering,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use tracing::{debug, error, info, level_filters::LevelFilter, trace};
use xml::{
    reader::{EventReader, XmlEvent},
    writer, EmitterConfig, EventWriter,
};
use xmltree::Element;

slint::include_modules!();

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
            let fleet = fleets_model.iter().nth(idx as usize).unwrap();
            trace!("Viewing fleet {}: {}", idx, fleet.name);
            let fleet_info_reader = FleetInfoReader::new(
                File::open(fleet.path.to_string())
                    .expect(&format!("Failed to open fleet {}", fleet.path.to_string())),
            );
            let description = fleet_info_reader.get_value("Fleet/Description");

            let main_window = main_window_weak.unwrap();
            main_window.set_cur_fleet_description(description.into());
            main_window.invoke_update_description();
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_save_description(move || {
            let main_window = main_window_weak.unwrap();
            let cur_description = main_window.get_cur_fleet_description();

            let cur_fleet_idx = main_window.get_cur_fleet_idx();
            let fleet = fleets_model.iter().nth(cur_fleet_idx as usize).unwrap();

            trace!("Opening fleet file");
            let fleet_file = File::open(&fleet.path).unwrap();

            trace!("Parsing fleet file");
            let Ok(mut element) = Element::parse(fleet_file).inspect_err(|err| {
                error!(%err, "Failed to parse fleet file");
                main_window.invoke_show_error_popup(
                    "Failed to parse fleet file".into(),
                    err.to_string().into(),
                );
            }) else {
                return;
            };

            if let Some(description_elem) = element.take_child("Description") {
                let text_node = description_elem
                    .children
                    .into_iter()
                    .next()
                    .unwrap_or(xmltree::XMLNode::Text(String::new()));
                let text = text_node.as_text().unwrap();
                trace!("Old description was: '{}'", text);
            }

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
            let fleet_file = OpenOptions::new()
                .write(true)
                .open(&fleet.path)
                .expect("Failed to open fleet file");
            element
                .write(fleet_file)
                .expect("Failed to write to fleet file");

            debug!("Fleet description saved");
        });
    }

    {
        let main_window_weak = main_window.as_weak();
        let fleets_model = fleets_model.clone();
        main_window.on_merge(move || {
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
            selected_fleets.iter().for_each(|fleet| {
                trace!("Pulling ships from fleet at '{}'", fleet.path);
                let file = File::open(fleet.path.to_string()).expect(&format!(
                    "Failed to open fleet '{}'",
                    fleet.path.to_string()
                ));

                Reader::new(EventReader::new(file), &mut ships).run_until_complete();
            });

            let main_window = main_window_weak.unwrap();
            let merge_output_name = main_window
                .get_merge_output_name()
                .to_string()
                .trim()
                .to_string();
            debug!("Merging fleets into '{}'", merge_output_name);
            if merge_output_name == "" {
                main_window.invoke_show_error_popup(
                    "No merge output name".into(),
                    "You must set an output name for the merged fleets".into(),
                );
                return;
            }

            let mut output = Vec::new();
            let mut writer = Writer::new(
                &mut output,
                EventReader::new(File::open(first_fleet.path.to_string()).expect(&format!(
                    "Failed to open primary fleet '{}'",
                    first_fleet.path
                ))),
                ships,
                merge_output_name,
            );
            writer.run_until_complete();
            debug!("Merge complete");
        });
    }

    main_window.run()?;

    Ok(())
}

type Ship = Vec<XmlEvent>;

struct FleetInfoReader<R: Read> {
    state: FleetInfoReaderState,
    event_reader: EventReader<R>,
    buf: String,
}
#[derive(PartialEq, Eq, Debug)]
enum FleetInfoReaderState {
    Idle,
    FindField(String, Vec<String>),
    ReadField,
    Complete,
}
impl<R: Read> FleetInfoReader<R> {
    fn new(reader: R) -> FleetInfoReader<R> {
        FleetInfoReader {
            state: FleetInfoReaderState::Idle,
            event_reader: EventReader::new(reader),
            buf: String::new(),
        }
    }

    fn get_value(mut self, arg: impl Into<String>) -> String {
        let mut fields: Vec<String> = arg.into().split("/").map(String::from).collect();
        self.state = FleetInfoReaderState::FindField(fields.remove(0), fields);
        while self.state != FleetInfoReaderState::Complete {
            self.tick();
        }
        self.buf
    }

    fn tick(&mut self) {
        let Ok(event) = self.event_reader.next() else {
            panic!("EventReader failed");
        };
        match &mut self.state {
            FleetInfoReaderState::Idle | FleetInfoReaderState::Complete => {}
            FleetInfoReaderState::FindField(field, remaining_fields) => match event {
                XmlEvent::StartElement { name, .. } if name.local_name.as_str() == field => {
                    self.state = if remaining_fields.is_empty() {
                        FleetInfoReaderState::ReadField
                    } else {
                        FleetInfoReaderState::FindField(
                            remaining_fields.remove(0),
                            remaining_fields.clone(),
                        )
                    };
                }
                // If a start element is followed by another start element, that means the
                // program has found an element that contains other elements; a list of elements.
                // Once we reach this, we terminate as we only want to read elements at the root
                // of the file.
                XmlEvent::StartElement { .. } => {
                    let Ok(event) = self.event_reader.next() else {
                        panic!("EventReader failed");
                    };
                    // Skip whitespace
                    let event = if let XmlEvent::Whitespace(_) = event {
                        let Ok(event) = self.event_reader.next() else {
                            panic!("EventReader failed");
                        };
                        event
                    } else {
                        event
                    };

                    if let XmlEvent::StartElement { .. } = event {
                        self.state = FleetInfoReaderState::Complete;
                        return;
                    } else {
                    }
                }
                _ => {}
            },
            FleetInfoReaderState::ReadField => match event {
                XmlEvent::CData(chunk) => {
                    self.buf.push_str(&chunk);
                }
                XmlEvent::EndElement { name: _ } => {
                    self.state = FleetInfoReaderState::Complete;
                }
                XmlEvent::Characters(chunk) => {
                    self.buf.push_str(&chunk);
                }
                _ => {}
            },
        }
    }
}

struct Reader<'a, R: Read> {
    state: ReadState,
    event_reader: EventReader<R>,
    current: Ship,
    emit_to: &'a mut Vec<Ship>,
}
enum ReadState {
    Start,
    ReadShips,
    ReadShip,
    Done,
    Error,
}
impl<'a, R: Read> Reader<'a, R> {
    fn new(event_reader: EventReader<R>, emit_to: &mut Vec<Ship>) -> Reader<R> {
        Reader {
            state: ReadState::Start,
            event_reader,
            current: Ship::new(),
            emit_to,
        }
    }

    fn run_until_complete(&mut self) {
        while !(self.is_done() || self.is_error()) {
            self.tick();
        }
    }

    fn tick(&mut self) {
        let Ok(event) = self.event_reader.next() else {
            self.state = ReadState::Error;
            return;
        };
        match self.state {
            ReadState::Start => match event {
                XmlEvent::StartElement { name, .. } if name.local_name.as_str() == "Ships" => {
                    self.state = ReadState::ReadShips;
                }
                _ => {}
            },
            ReadState::ReadShips => match event.clone() {
                XmlEvent::EndElement { .. } => {
                    self.state = ReadState::Done;
                }
                XmlEvent::StartElement { name, .. } if name.local_name.as_str() == "Ship" => {
                    self.current.push(event);
                    self.state = ReadState::ReadShip;
                }
                _ => {}
            },
            ReadState::ReadShip => {
                self.current.push(event.clone());
                match &event {
                    XmlEvent::EndElement { name } if name.local_name.as_str() == "Ship" => {
                        let cur = std::mem::replace(&mut self.current, Vec::new());
                        self.emit_to.push(cur);
                        self.state = ReadState::ReadShips;
                    }
                    _ => {}
                }
            }
            ReadState::Done => {}
            ReadState::Error => {}
        }
    }
    fn is_done(&self) -> bool {
        match self.state {
            ReadState::Done => true,
            _ => false,
        }
    }
    fn is_error(&self) -> bool {
        match self.state {
            ReadState::Done => true,
            _ => false,
        }
    }
}

struct Writer<W: Write, R: Read> {
    event_writer: xml::writer::EventWriter<W>,
    main: EventReader<R>,
    insert: Vec<Ship>,
    state: WriteState,
    name: String,
}
#[derive(Debug, Clone, Copy)]
enum WriteState {
    Start,
    Inserting,
    Finishing,
    Done,
    Error,
}
impl<W: Write, R: Read> Writer<W, R> {
    fn new(writer: W, main: EventReader<R>, insert: Vec<Ship>, name: String) -> Writer<W, R> {
        Writer {
            event_writer: EventWriter::new_with_config(
                writer,
                EmitterConfig::new().perform_indent(true),
            ),
            main,
            insert,
            state: WriteState::Start,
            name,
        }
    }

    fn run_until_complete(&mut self) {
        trace!("Copying primary fleet");
        while !(self.is_done() || self.is_error()) {
            self.tick();
        }
    }

    fn tick(&mut self) {
        match self.state {
            WriteState::Start => {
                let Ok(event) = self.main.next() else {
                    self.state = WriteState::Error;
                    return;
                };
                self.write_event(event.clone());
                match event {
                    XmlEvent::StartElement { name, .. } if name.local_name.as_str() == "Name" => {
                        trace!("Overwriting fleet name");
                        let _old_name = self.main.skip();
                        self.event_writer
                            .write(writer::XmlEvent::Characters(&self.name))
                            .unwrap();
                        self.event_writer
                            .write(writer::XmlEvent::EndElement {
                                name: Some(name.borrow()),
                            })
                            .unwrap();
                    }
                    XmlEvent::StartElement { name, .. } if name.local_name.as_str() == "Ships" => {
                        trace!("Injecting ships");
                        self.state = WriteState::Inserting;
                    }
                    _ => {}
                }
            }
            WriteState::Inserting => {
                let ships = std::mem::replace(&mut self.insert, Vec::new());
                ships
                    .into_iter()
                    .map(|ship| ship.into_iter())
                    .flatten()
                    .for_each(|event| {
                        self.write_event(event);
                    });
                trace!("Finishing fleet file");
                self.state = WriteState::Finishing;
            }
            WriteState::Finishing => {
                let Ok(event) = self.main.next() else {
                    self.state = WriteState::Error;
                    return;
                };
                self.write_event(event.clone());
                match event {
                    XmlEvent::EndDocument => {
                        self.state = WriteState::Done;
                    }
                    _ => {}
                }
            }
            WriteState::Done => {}
            WriteState::Error => {}
        }
    }

    fn write_event(&mut self, event: XmlEvent) {
        // println!("{:?}", self.state);
        // println!("{:?}", &event);
        // println!();
        let event = match &event {
            XmlEvent::StartDocument {
                version,
                standalone,
                ..
            } => Some(writer::XmlEvent::StartDocument {
                version: version.clone(),
                encoding: Some("UTF-8"),
                standalone: standalone.clone(),
            }),
            XmlEvent::EndDocument => None,
            XmlEvent::ProcessingInstruction { name, data } => {
                Some(writer::XmlEvent::ProcessingInstruction {
                    name: &name,
                    data: data.as_ref().map(|s| s.as_str()),
                })
            }
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => Some(writer::XmlEvent::StartElement {
                name: name.borrow(),
                attributes: attributes.iter().map(|attr| attr.borrow()).collect(),
                namespace: namespace.borrow(),
            }),
            XmlEvent::EndElement { name } => Some(writer::XmlEvent::EndElement {
                name: Some(name.borrow()),
            }),
            XmlEvent::CData(s) => Some(writer::XmlEvent::CData(&s)),
            XmlEvent::Comment(s) => Some(writer::XmlEvent::Comment(&s)),
            XmlEvent::Characters(s) => Some(writer::XmlEvent::Characters(&s)),
            XmlEvent::Whitespace(_) => None,
        };
        if let Some(event) = event {
            self.event_writer.write(event).unwrap();
        }
    }

    fn is_done(&self) -> bool {
        match self.state {
            WriteState::Done => true,
            _ => false,
        }
    }
    fn is_error(&self) -> bool {
        match self.state {
            WriteState::Error => true,
            _ => false,
        }
    }
}
