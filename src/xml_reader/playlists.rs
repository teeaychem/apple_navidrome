use xml::common::Position;
use xml::reader::XmlEvent;

use crate::{
    structs::{playlist::Playlist, TrackID},
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
                // copies for error reporting, while allowing ok matches to take ownership
                let the_key = name.local_name.clone();
                let the_position = reader.parser.position();
                match name.local_name.as_str() {
                    "key" => {
                        let key = reader.element_as_string(Some("key"))?;
                        if key == "Playlist Items" {
                            the_playlist.track_ids = playlist_ids(reader)?;
                        } else {
                            let value = reader.element_as_string(None)?;
                            match key.as_str() {
                                "Name" => the_playlist.name = value,
                                "Description" => the_playlist.description = value,
                                "Playlist Persistent ID" => the_playlist.persistent_id = value,
                                "Parent Persistent ID" => the_playlist.parent_persistent_id = value,
                                "Folder" => match value.as_str() {
                                    "true" => the_playlist.folder = true,
                                    "false" => the_playlist.folder = false,
                                    _ => {
                                        return Err(err::LibraryXmlReader::ExpectedBooleanTag {
                                            position: reader.parser.position(),
                                        })
                                    }
                                },
                                "Master" => {}
                                "Playlist ID" => {}
                                "Smart Info" => {}
                                "Smart Criteria" => {}
                                "Distinguished Kind" => {}
                                "Music" => {}
                                "Visible" => {}
                                "All Items" => {}
                                _ => {
                                    log::error!("Playlist parsing failed");
                                    return Err(err::LibraryXmlReader::UnexpectedKey {
                                        position: the_position,
                                        key: the_key,
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(err::LibraryXmlReader::UnexpectedKey {
                            position: reader.parser.position(),
                            key: the_key,
                        })
                    }
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
) -> Result<Vec<TrackID>, xml_reader::err::LibraryXmlReader> {
    let mut ids = Vec::default();
    reader.eat_start("array")?;
    loop {
        match reader.peek() {
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == "dict" {
                    reader.eat_start("dict")?;
                    let _key = reader.element_as_string(Some("key"));
                    let id = reader.element_as_string(Some("integer"))?;
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
                        _ => {
                            return Err(err::LibraryXmlReader::UnexpectedKey {
                                position: reader.parser.position(),
                                key: name.local_name.to_owned(),
                            })
                        }
                    }
                }
                XmlEvent::EndElement { .. } => {
                    reader.eat_end("array")?;
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
