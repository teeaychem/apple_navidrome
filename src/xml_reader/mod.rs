use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::common::{Position, TextPosition};

use xml::reader::{EventReader, ParserConfig2, XmlEvent};

use crate::structs::track::TrackErr;
use crate::structs::Library;

pub mod playlists;
pub mod tracks;

pub struct LibraryXmlReader {
    pub parser: EventReader<BufReader<File>>,
    pub event: XmlEvent,
}

pub mod err {
    use crate::structs::TrackID;

    use super::*;

    #[allow(dead_code)]
    #[derive(Debug)]
    pub enum LibraryXmlReader {
        Io {
            error: std::io::Error,
        },
        Xml {
            error: xml::reader::Error,
        },
        KeyNotStart {
            position: TextPosition,
        },
        UnexpectedElement {
            position: TextPosition,
        },
        UnexpectedKey {
            position: TextPosition,
            key: String,
        },
        UnableToEat {
            position: TextPosition,
            want: String,
        },
        BadEat {
            position: TextPosition,
            want: String,
            got: String,
        },
        BadPair {
            position: TextPosition,
            start: String,
            end: String,
        },
        MissingTrack {
            playlist: String,
            track_id: TrackID,
        },
        Track(TrackErr),
    }
}

impl From<xml::reader::Error> for err::LibraryXmlReader {
    fn from(error: xml::reader::Error) -> Self {
        err::LibraryXmlReader::Xml { error }
    }
}

impl From<std::io::Error> for err::LibraryXmlReader {
    fn from(error: std::io::Error) -> Self {
        err::LibraryXmlReader::Io { error }
    }
}

impl From<TrackErr> for err::LibraryXmlReader {
    fn from(error: TrackErr) -> Self {
        err::LibraryXmlReader::Track(error)
    }
}

impl LibraryXmlReader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, err::LibraryXmlReader> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut parser = ParserConfig2::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .coalesce_characters(true)
            .create_reader(reader);
        let next = parser.next()?;
        Ok(LibraryXmlReader {
            parser,
            event: next,
        })
    }

    pub fn peek(&self) -> &XmlEvent {
        &self.event
    }

    pub fn forward(&mut self) -> Result<&XmlEvent, xml::reader::Error> {
        self.event = self.parser.next()?;
        Ok(&self.event)
    }

    pub fn eat_start(&mut self, name: &str) -> Result<(), err::LibraryXmlReader> {
        let XmlEvent::StartElement { name: found, .. } = self.peek() else {
            return Err(err::LibraryXmlReader::UnableToEat {
                position: self.parser.position(),
                want: name.to_string(),
            });
        };
        if found.local_name != name {
            return Err(err::LibraryXmlReader::BadEat {
                position: self.parser.position(),
                want: found.to_string(),
                got: name.to_string(),
            });
        }
        self.event = self.parser.next()?;
        Ok(())
    }

    pub fn eat_end(&mut self, name: &str) -> Result<(), err::LibraryXmlReader> {
        let XmlEvent::EndElement { name: found, .. } = self.peek() else {
            return Err(err::LibraryXmlReader::UnableToEat {
                position: self.parser.position(),
                want: name.to_string(),
            });
        };
        if found.local_name != name {
            return Err(err::LibraryXmlReader::BadEat {
                position: self.parser.position(),
                want: found.to_string(),
                got: name.to_string(),
            });
        }
        self.event = self.parser.next()?;
        Ok(())
    }

    pub fn element_as_string(
        &mut self,
        name: Option<&str>,
    ) -> Result<String, err::LibraryXmlReader> {
        let XmlEvent::StartElement { name: start, .. } = self.peek() else {
            return Err(err::LibraryXmlReader::KeyNotStart {
                position: self.parser.position(),
            });
        };
        let element = start.local_name.clone();
        if let Some(name_check) = name {
            if start.local_name != name_check {
                return Err(err::LibraryXmlReader::BadEat {
                    position: self.parser.position(),
                    want: name_check.to_owned(),
                    got: start.local_name.to_owned(),
                });
            }
        }
        let as_string = match element.as_str() {
            "true" => {
                let _ = self.forward();
                element.clone()
            }
            "false" => {
                let _ = self.forward();
                element.clone()
            }
            "string" => {
                //
                self.forward()?;
                match self.peek() {
                    XmlEvent::Characters(chars) => {
                        let the_string = chars.to_owned();
                        self.forward()?;
                        the_string
                    }
                    _ => "".to_owned(),
                }
            }
            _ => {
                let XmlEvent::Characters(chars) = self.forward()? else {
                    return Err(err::LibraryXmlReader::UnexpectedKey {
                        position: self.parser.position(),
                        key: element,
                    });
                };

                let the_string = chars.to_owned();
                self.forward()?;
                the_string
            }
        };

        let XmlEvent::EndElement { name: end, .. } = self.peek() else {
            return Err(err::LibraryXmlReader::UnexpectedElement {
                position: self.parser.position(),
            });
        };
        if end.local_name != element {
            return Err(err::LibraryXmlReader::BadPair {
                position: self.parser.position(),
                start: element,
                end: end.local_name.clone(),
            });
        }
        if let Some(name_check) = name {
            if end.local_name != name_check {
                return Err(err::LibraryXmlReader::BadEat {
                    position: self.parser.position(),
                    want: name_check.to_owned(),
                    got: end.local_name.to_owned(),
                });
            }
        }
        self.forward()?;

        Ok(as_string)
    }
}

pub fn build_library(path: &Path) -> Result<Library, crate::xml_reader::err::LibraryXmlReader> {
    let mut the_lib = Library::default();
    let mut reader = LibraryXmlReader::new(path).unwrap();
    // skip until library dictionary
    loop {
        if let Ok(xml::reader::XmlEvent::StartElement { name, .. }) = reader.forward() {
            if name.local_name == "dict" {
                break;
            }
        }
    }
    reader.eat_start("dict")?;
    loop {
        match reader.peek() {
            xml::reader::XmlEvent::StartElement { name, .. } => {
                if name.local_name == "key" {
                    let key = reader.element_as_string(Some("key")).unwrap();
                    match key.as_str() {
                        "Tracks" => the_lib.import_tracks(&mut reader)?,
                        "Playlists" => the_lib.import_playlists(&mut reader)?,
                        _ => {
                            print!("{key} : ");
                            let value = reader.element_as_string(None).unwrap();
                            println!("{value}");
                        }
                    }
                } else {
                    panic!(
                        "{} :Unexpected xml start element {name}",
                        xml::common::Position::position(&reader.parser)
                    );
                }
            }
            xml::reader::XmlEvent::EndElement { .. } => {
                reader.eat_end("dict")?;
                break;
            }
            _ => {
                panic!(
                    "{} : Unexpected xml event {:?}",
                    xml::common::Position::position(&reader.parser),
                    reader.peek()
                );
            }
        }
    }
    Ok(the_lib)
}
