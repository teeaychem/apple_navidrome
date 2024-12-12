use chrono::{DateTime, Utc};
use std::{num::ParseIntError, time::Duration};

use xml::{common::Position, reader::XmlEvent};

use crate::{
    structs::{
        track::{Track, TrackErr},
        Library, TrackID,
    },
    xml_reader::{self},
};

use super::LibraryXmlReader;

impl From<chrono::ParseError> for TrackErr {
    fn from(error: chrono::ParseError) -> Self {
        TrackErr::ParseDateTime(error)
    }
}

impl From<ParseIntError> for TrackErr {
    fn from(error: ParseIntError) -> Self {
        TrackErr::ParseInt(error)
    }
}

#[rustfmt::skip]
pub fn get_track(reader: &mut xml_reader::LibraryXmlReader) -> Result<Track, TrackErr> {
    let _ = reader.forward();
    let mut the_track = Track::default();
    loop {
        match reader.peek() {
            XmlEvent::StartElement { .. } => {
                let key = reader.element_as_string(Some("key")).unwrap();
                let value = reader.element_as_string(None).unwrap();
                match key.as_str() {

                    "Album Artist" => the_track.album_artist = Some(value),
                    "Album Rating Computed" => {}
                    "Album Rating" => the_track.album_rating = Some(value.parse::<usize>()?),
                    "Album" => the_track.album_title = Some(value),
                    "Artist" => the_track.artist = Some(value),
                    "Artwork Count" => {}
                    "BPM" => the_track.bpm = Some(value.parse::<usize>()?),
                    "Bit Rate" => {}
                    "Comments" => the_track.comments = Some(value),
                    "Compilation" => the_track.compiltion = true,
                    "Composer" => the_track.composer = Some(value),
                    "Date Added" => the_track.date_added = value.parse::<DateTime<Utc>>()?,
                    "Date Modified" => the_track.date_modified = value.parse::<DateTime<Utc>>()?,
                    "Disabled" => {}
                    "Disc Count" => the_track.disc_count = Some(value.parse::<usize>()?),
                    "Disc Number" => the_track.disc_number = Some(value.parse::<usize>()?),
                    "Favorited" => the_track.favourited = true,
                    "File Folder Count" => {}
                    "Genre" => the_track.genre = Some(value),
                    "Grouping" => the_track.grouping = Some(value),
                    "Kind" => {}
                    "Library Folder Count" => {}
                    "Location" => the_track.location = value,
                    "Loved" => the_track.loved = true,
                    "Movement Count" => {}
                    "Movement Name" => the_track.movement_title = Some(value),
                    "Movement Number" => the_track.movement_number = Some(value.parse::<usize>()?),
                    "Name" => the_track.title = Some(value),
                    "Normalization" => {}
                    "Part Of Gapless Album" => {}
                    "Persistent ID" => the_track.persistent_id = value,
                    "Play Count" => the_track.play_count = value.parse::<usize>()?,
                    "Play Date UTC" => the_track.play_date = Some(value.parse::<DateTime<Utc>>()?),
                    "Play Date" => {} // use utc variant
                    "Rating Computed" => {}
                    "Rating" => the_track.rating = value.parse::<usize>()?,
                    "Release Date" => the_track.release_data = Some(value.parse::<DateTime<Utc>>()?),
                    "Sample Rate" => {}
                    "Size" => the_track.size = value.parse::<usize>()?,
                    "Skip Count" => the_track.skip_count = value.parse::<usize>()?,
                    "Skip Date" => the_track.skip_date = Some(value.parse::<DateTime<Utc>>()?),
                    "Sort Album Artist" => {}
                    "Sort Album" => {}
                    "Sort Artist" => {}
                    "Sort Composer" => {}
                    "Sort Name" => {}
                    "Total Time" => the_track.duration = Duration::from_millis(value.parse::<u64>()?),
                    "Track Count" => the_track.total_tracks = Some(value.parse::<usize>()?),
                    "Track ID" => the_track.id = value.parse::<TrackID>()?,
                    "Track Number" => the_track.track_number = Some(value.parse::<usize>()?),
                    "Track Type" => {}
                    "Volume Adjustment" => {}
                    "Work" => the_track.work = Some(value),
                    "Year" => the_track.year = Some(value.parse::<usize>()?),

                    // missed something?
                    _ => {
                        let title = the_track.title.clone().unwrap_or("[No title]".to_string());
                        let artist = the_track
                            .artist
                            .clone()
                            .unwrap_or("[No artist]".to_string());
                        println!("Unexpected key:\n\t\"{key}\" with value\n\t\"{value}\"\nwhen reading{title} by{artist}.");
                    }
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "dict" {
                    let _ = reader.forward();
                    break;
                } else {
                    panic!();
                }
            }
            _ => {}
        }
    }
    Ok(the_track)
}

impl Library {
    pub fn import_tracks(
        &mut self,
        reader: &mut LibraryXmlReader,
    ) -> Result<(), xml_reader::err::LibraryXmlReader> {
        // process each track
        reader.eat_start("dict")?;
        loop {
            match reader.peek() {
                XmlEvent::StartElement { name, .. } => {
                    //
                    if name.local_name == "key" {
                        let id = reader
                            .element_as_string(Some("key"))
                            .unwrap()
                            .parse::<usize>()
                            .expect("id?");
                        let track = get_track(reader)?;
                        assert_eq!(id, track.id);
                        self.tracks.insert(id, track);
                    } else {
                        panic!("Failed to process track {}", reader.parser.position());
                    }
                }
                XmlEvent::EndElement { .. } => {
                    reader.eat_end("dict")?;
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
