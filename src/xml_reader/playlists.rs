use std::fs::File;
use std::io::Write;
use std::path::Path;
use xml::common::Position;
use xml::reader::XmlEvent;

use crate::{
    structs::{playlist::Playlist, TrackID, TrackMap},
    xml_reader::{self},
};

use super::*;

pub fn get_playlist(
    reader: &mut LibraryXmlReader,
) -> Result<Playlist, xml_reader::err::LibraryXmlReader> {
    let mut the_playlist = Playlist::default();
    reader.eat_start("dict")?;
    loop {
        match reader.peek() {
            XmlEvent::StartElement { name, .. } => {
                //
                match name.local_name.as_str() {
                    "key" => {
                        let key = reader.element_as_string(Some("key")).unwrap();
                        if key == "Playlist Items" {
                            the_playlist.track_ids = playlist_ids(reader)?;
                        } else {
                            let value = reader.element_as_string(None).unwrap();
                            match key.as_str() {
                                "Name" => the_playlist.name = value,
                                "Description" => the_playlist.description = value,
                                "Playlist Persistent ID" => the_playlist.persistent_id = value,
                                "Parent Persistent ID" => the_playlist.parent_persistent_id = value,
                                "Folder" => match value.as_str() {
                                    "true" => the_playlist.folder = true,
                                    "false" => the_playlist.folder = false,
                                    _ => panic!("Unexpected"),
                                },
                                "Master" | "Playlist ID" | "Smart Info" | "Smart Criteria"
                                | "Distinguished Kind" | "Music" | "Visible" | "All Items" => {
                                    // skip these
                                }
                                _ => panic!(
                                    "{} : Playlist parsing failed ({key})",
                                    reader.parser.position()
                                ),
                            }
                        }
                    }
                    _ => panic!("Failed to process track {}", reader.parser.position()),
                }
            }
            XmlEvent::EndElement { .. } => {
                reader.eat_end("dict")?;
                break;
            }
            _ => {}
        }
    }
    Ok(the_playlist)
}

pub fn playlist_ids(
    reader: &mut LibraryXmlReader,
) -> Result<Vec<usize>, xml_reader::err::LibraryXmlReader> {
    let mut ids = Vec::default();
    reader.eat_start("array")?;
    loop {
        match reader.peek() {
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == "dict" {
                    reader.eat_start("dict")?;
                    let _key = reader.element_as_string(Some("key"));
                    let id = reader
                        .element_as_string(Some("integer"))
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    ids.push(id);
                    reader.eat_end("dict")?;
                }
            }
            XmlEvent::EndElement { .. } => {
                reader.eat_end("array")?;
                break;
            }
            _ => {}
        }
    }
    Ok(ids)
}

impl Library {
    pub fn import_playlists(
        &mut self,
        reader: &mut LibraryXmlReader,
    ) -> Result<(), xml_reader::err::LibraryXmlReader> {
        reader.eat_start("array")?;
        // process each track
        loop {
            match reader.peek() {
                XmlEvent::StartElement { name, .. } => {
                    //
                    match name.local_name.as_str() {
                        "dict" => {
                            self.playlists.push(get_playlist(reader)?);
                        }
                        _ => panic!("Failed to process track {}", reader.parser.position()),
                    }
                }
                XmlEvent::EndElement { .. } => {
                    //
                    reader.eat_end("array")?;
                    break;
                }

                XmlEvent::Characters(chars) => {
                    panic!("Found chars {chars}");
                }

                _ => {}
            }
        }
        Ok(())
    }
}
