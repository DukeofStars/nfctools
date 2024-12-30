use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
};

use slint::{Model, SharedString, VecModel, Weak};
use tracing::{debug, trace};
use xml::{
    name::OwnedName, namespace::Namespace, reader::XmlEvent, writer, EmitterConfig, EventReader,
    EventWriter,
};

use crate::{error::wrap_errorable_function, my_error, FleetData, MainWindow};

type Ship = Vec<XmlEvent>;
type MissileType = Vec<XmlEvent>;

pub fn on_merge_handler(
    main_window_weak: Weak<MainWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn(SharedString) {
    move |merge_output_name| {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
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
    }
}

pub struct Reader<'a, R: Read> {
    state: ReadState,
    event_reader: EventReader<R>,
    current: Ship,
    emit_to: &'a mut Vec<Ship>,
    element_name: String,
    indiv_element_name: String,
}
enum ReadState {
    Start,
    ReadElements,
    ReadElement,
    Done,
    Error,
}
impl<'a, R: Read> Reader<'a, R> {
    pub fn new(
        event_reader: EventReader<R>,
        emit_to: &mut Vec<Ship>,
        element_name: impl Into<String>,
        indiv_element_name: impl Into<String>,
    ) -> Reader<R> {
        Reader {
            state: ReadState::Start,
            event_reader,
            current: Ship::new(),
            emit_to,
            element_name: element_name.into(),
            indiv_element_name: indiv_element_name.into(),
        }
    }

    pub fn run_until_complete(&mut self) {
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
                XmlEvent::StartElement { name, .. } if name.local_name == self.element_name => {
                    self.state = ReadState::ReadElements;
                }
                XmlEvent::EndDocument => {
                    self.state = ReadState::Done;
                }
                _ => {}
            },
            ReadState::ReadElements => match event.clone() {
                XmlEvent::EndElement { .. } => {
                    self.state = ReadState::Done;
                }
                XmlEvent::StartElement { name, .. }
                    if name.local_name == self.indiv_element_name =>
                {
                    self.current.push(event);
                    self.state = ReadState::ReadElement;
                }
                _ => {}
            },
            ReadState::ReadElement => {
                self.current.push(event.clone());
                match &event {
                    XmlEvent::EndElement { name } if name.local_name == self.indiv_element_name => {
                        let cur = std::mem::replace(&mut self.current, Vec::new());
                        self.emit_to.push(cur);
                        self.state = ReadState::ReadElements;
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

pub struct Writer<W: Write, R: Read> {
    event_writer: xml::writer::EventWriter<W>,
    main: EventReader<R>,
    insert_ships: Vec<Ship>,
    insert_missiles: Vec<MissileType>,
    state: WriteState,
    name: String,
}
#[derive(Debug, Clone, Copy)]
enum WriteState {
    FindShips,
    InsertingShips,
    FindMissiles,
    InsertingMissiles,
    Finishing,
    Done,
    Error,
}
impl<W: Write, R: Read> Writer<W, R> {
    pub fn new(
        writer: W,
        main: EventReader<R>,
        insert_ships: Vec<Ship>,
        insert_missiles: Vec<MissileType>,
        name: String,
    ) -> Writer<W, R> {
        Writer {
            event_writer: EventWriter::new_with_config(
                writer,
                EmitterConfig::new().perform_indent(true),
            ),
            main,
            insert_ships,
            insert_missiles,
            state: WriteState::FindShips,
            name,
        }
    }

    pub fn run_until_complete(&mut self) {
        trace!("Copying primary fleet");
        while !(self.is_done() || self.is_error()) {
            self.tick();
        }
    }

    fn tick(&mut self) {
        match self.state {
            WriteState::FindShips => {
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
                        self.state = WriteState::InsertingShips;
                    }
                    _ => {}
                }
            }
            WriteState::InsertingShips => {
                let ships = std::mem::replace(&mut self.insert_ships, Vec::new());
                ships
                    .into_iter()
                    .map(|ship| ship.into_iter())
                    .flatten()
                    .for_each(|event| {
                        self.write_event(event);
                    });
                self.state = WriteState::FindMissiles;
            }
            WriteState::FindMissiles => {
                let Ok(event) = self.main.next() else {
                    self.state = WriteState::Error;
                    return;
                };
                match event.clone() {
                    XmlEvent::StartElement { name, .. }
                        if name.local_name.as_str() == "MissileTypes" =>
                    {
                        trace!("Injecting missiles");
                        self.state = WriteState::InsertingMissiles;
                    }
                    // If the fleet element ends, that means the primary fleet has no missile types.
                    // We must insert the missile types into the document.
                    XmlEvent::EndElement { name } if name.local_name.as_str() == "Fleet" => {
                        // Write the starting tag
                        self.write_event(XmlEvent::StartElement {
                            name: OwnedName::local("MissileTypes"),
                            attributes: Vec::new(),
                            namespace: Namespace::empty(),
                        });
                        // Inject the missiles
                        let missiles = std::mem::replace(&mut self.insert_missiles, Vec::new());
                        missiles
                            .into_iter()
                            .map(|missile| missile.into_iter())
                            .flatten()
                            .for_each(|event| {
                                self.write_event(event);
                            });
                        // Write the ending tag
                        self.write_event(XmlEvent::EndElement {
                            name: OwnedName::local("MissileTypes"),
                        });
                        // Switch state
                        trace!("Finishing fleet file");
                        self.state = WriteState::Finishing;
                    }
                    _ => {}
                }
                self.write_event(event);
            }
            WriteState::InsertingMissiles => {
                let missiles = std::mem::replace(&mut self.insert_missiles, Vec::new());
                missiles
                    .into_iter()
                    .map(|missile| missile.into_iter())
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
