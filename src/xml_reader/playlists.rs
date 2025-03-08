use xml::common::Position;
use xml::reader::XmlEvent;

use crate::{
    structs::{playlist::Playlist, TrackID},
    xml_reader::{self, err},
};

use super::*;

/// (Optional) List of playlist keys that you might want to ignore; not used below.
// const IGNORED_KEYS: &[&str] = &[
//     "Purchased",
//     "Explicit",
//     "Playlist Only",
//     "Disliked",
//     "Sort Series",
//     "Clean",
// ];

pub fn get_playlist(
    reader: &mut LibraryXmlReader,
) -> Result<Playlist, xml_reader::err::LibraryXmlReader> {
    let mut the_playlist = Playlist::default();
    reader.eat_start("dict")?;
    loop {
        match reader.peek() {
            XmlEvent::StartElement { name, .. } => {
                // Copy key name for logging.
                let the_key = name.local_name.clone();
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
                                "Master" => {},
                                "Playlist ID" => {},
                                "Smart Info" => {},
                                "Smart Criteria" => {},
                                "Distinguished Kind" => {},
                                "Music" => {},
                                "Visible" => {},
                                "All Items" => {},
                                _ => {
                                    log::debug!("Ignoring playlist key '{}' with value '{}'", key, value);
                                    continue; // Skip unknown key/value pair
                                }
                            }
                        }
                    }
                    _ => {
                        log::debug!("Ignoring unexpected element '{}' in playlist dict", the_key);
                        let _ = reader.forward();
                        continue;
                    }
                }
            }
            XmlEvent::EndElement { .. } => {
                reader.eat_end("dict")?;
                break;
            }
            _ => { let _ = reader.forward(); continue; }
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
                    let _ = reader.element_as_string(Some("key"));
                    let id = reader.element_as_string(Some("integer"))?;
                    ids.push(id);
                    reader.eat_end("dict")?;
                } else {
                    let _ = reader.forward();
                }
            }
            XmlEvent::EndElement { .. } => {
                reader.eat_end("array")?;
                break;
            }
            _ => { let _ = reader.forward(); continue; }
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
        // Process each playlist in the playlists array.
        loop {
            match reader.peek() {
                XmlEvent::StartElement { name, .. } => {
                    match name.local_name.as_str() {
                        "dict" => {
                            self.playlists.push(get_playlist(reader)?);
                        }
                        _ => {
                            log::debug!("Ignoring unexpected element '{}' in playlists array", name.local_name);
                            let _ = reader.forward();
                            continue;
                        }
                    }
                }
                XmlEvent::EndElement { .. } => {
                    reader.eat_end("array")?;
                    break;
                }
                _ => { let _ = reader.forward(); continue; }
            }
        }
        Ok(())
    }
}
