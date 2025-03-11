use xml::common::Position;
use xml::reader::XmlEvent;

use crate::{
    structs::{playlist::Playlist, TrackID},
    xml_reader::{self, err},
};

use super::*;

/// Reads a [Playlist] from `reader`, if possible.
///
/// Follows the same pattern as [get_track] with respect to keys.
/// Specifically, known keys are either used or ignored, while unknown keys are logged.
pub fn get_playlist(
    reader: &mut LibraryXmlReader,
) -> Result<Playlist, xml_reader::err::LibraryXmlReader> {
    let mut playlist = Playlist::default();
    reader.eat_start("dict")?;
    loop {
        match reader.peek() {
            XmlEvent::StartElement { name, .. } => {
                // Copy key name for logging.
                let key = name.local_name.clone();
                match name.local_name.as_str() {
                    "key" => {
                        let key = reader.element_as_string(Some("key"))?;
                        if key == "Playlist Items" {
                            playlist.track_ids = playlist_ids(reader)?;
                        } else {
                            let value = reader.element_as_string(None)?;
                            match key.as_str() {
                                "Description" => playlist.description = value,
                                "All Items" => {}
                                "Distinguished Kind" => {}
                                "Folder" => match value.as_str() {
                                    "true" => playlist.folder = true,
                                    "false" => playlist.folder = false,
                                    _ => {
                                        return Err(err::LibraryXmlReader::ExpectedBooleanTag {
                                            position: reader.parser.position(),
                                        })
                                    }
                                },
                                "Master" => {}
                                "Music" => {}
                                "Name" => playlist.name = value,
                                "Parent Persistent ID" => playlist.parent_persistent_id = value,
                                "Playlist ID" => {}
                                "Playlist Persistent ID" => playlist.persistent_id = value,
                                "Smart Criteria" => {}
                                "Smart Info" => {}
                                "Visible" => {}
                                // Skip unknown key/value pair
                                _ => {
                                    log::debug!(
                                        "Ignoring playlist key '{key}' with value '{value}'"
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                    _ => {
                        log::debug!("Ignoring unexpected element '{key}' in playlist dict");
                        let _ = reader.forward();
                        continue;
                    }
                }
            }
            XmlEvent::EndElement { .. } => {
                reader.eat_end("dict")?;
                break;
            }
            _ => {
                log::debug!(
                    "Ignoring unexpected element '{:?}' in playlist dict",
                    reader.peek()
                );
                let _ = reader.forward();
                continue;
            }
        }
    }
    Ok(playlist)
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
            _ => {
                log::debug!(
                    "Ignoring unexpected element '{:?}' when examining playlist ids",
                    reader.peek()
                );
                let _ = reader.forward();
                continue;
            }
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
                XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                    "dict" => {
                        self.playlists.push(get_playlist(reader)?);
                    }
                    _ => {
                        log::debug!(
                            "Ignoring unexpected element '{}' in playlists array",
                            name.local_name
                        );
                        let _ = reader.forward();
                        continue;
                    }
                },
                XmlEvent::EndElement { .. } => {
                    reader.eat_end("array")?;
                    break;
                }
                _ => {
                    log::debug!(
                        "Ignoring unexpected element '{:?}' in playlist array",
                        reader.peek()
                    );
                    let _ = reader.forward();
                    continue;
                }
            }
        }
        Ok(())
    }
}
