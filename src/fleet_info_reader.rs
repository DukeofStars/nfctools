use std::io::Read;

use xml::{reader::XmlEvent, EventReader};

pub struct FleetInfoReader<R: Read> {
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
    pub fn new(reader: R) -> FleetInfoReader<R> {
        FleetInfoReader {
            state: FleetInfoReaderState::Idle,
            event_reader: EventReader::new(reader),
            buf: String::new(),
        }
    }

    pub fn get_value(mut self, arg: impl Into<String>) -> String {
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
