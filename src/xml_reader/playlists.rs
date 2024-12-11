use std::fs::File;
use std::io::Write;
use std::path::Path;
use xml::common::Position;
use xml::reader::XmlEvent;

use crate::{structs::{TrackID, TrackMap}, xml_reader::{self}};

use super::*;

#[derive(Default, Debug)]
pub struct Playlist {
    pub name: String,
    pub description: String,
    pub persistent_id: String,
    pub parent_persistent_id: String,
    pub folder: bool,
    pub track_ids: Vec<TrackID>,
}

impl Playlist {
    pub fn export_m3u8(
        &self,
        path: &Path,
        tracks: &TrackMap,
    ) -> Result<(), xml_reader::err::LibraryXmlReader> {
        let playlist_filename = format!("{}.m3u8", self.name);
        let playlist_path = path.join(Path::new(&playlist_filename));
        match File::create(playlist_path.clone()) {
            Ok(mut file) => {
                writeln!(file, "#EXTM3U")?;
                writeln!(file, "#EXTENC:UTF-8")?;
                writeln!(file, "#PLAYLIST:{}", self.name)?;
                for id in &self.track_ids {
                    let track = match tracks.get(id) {
                        Some(t) => t,
                        None => {
                            return Err(xml_reader::err::LibraryXmlReader::MissingTrack {
                                playlist: self.name.to_owned(),
                                track_id: *id,
                            })
                        }
                    };

                    writeln!(file, "#EXTINF:{},{} - {}",
                        track.duration.as_secs(),
                        track.artist,
                        track.title)?;
                    let abs_pth = &track.location;
                    writeln!(file, "{}", abs_pth)?;
                }
            }
            Err(_) => {
                println!("Failed to create a file for playlist {}", self.name);
            }
        }
        Ok(())
    }
}

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

pub fn get_playlists(
    reader: &mut LibraryXmlReader,
) -> Result<Vec<Playlist>, xml_reader::err::LibraryXmlReader> {
    let mut the_lists = Vec::default();
    reader.eat_start("array")?;
    // process each track
    loop {
        match reader.peek() {
            XmlEvent::StartElement { name, .. } => {
                //
                match name.local_name.as_str() {
                    "dict" => {
                        the_lists.push(get_playlist(reader)?);
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
    Ok(the_lists)
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
