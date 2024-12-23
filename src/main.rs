use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use xml::{
    reader::{EventReader, XmlEvent},
    writer, EmitterConfig, EventWriter,
};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Merge {
        fleet1: PathBuf,
        fleet2: PathBuf,
        out: PathBuf,
        #[clap(short, long)]
        name: String,
    },
}

slint::include_modules!();

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let main_window = MainWindow::new()?;

    main_window.run()?;

    Ok(())
}

type Ship = Vec<XmlEvent>;

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
