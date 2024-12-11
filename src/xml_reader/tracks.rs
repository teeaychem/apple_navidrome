use chrono::{DateTime, Utc};
use std::{collections::HashMap, num::ParseIntError, time::Duration};

use xml::{common::Position, reader::XmlEvent};

use crate::{structs::{track::{Track, TrackErr}, TrackID, TrackMap}, xml_reader::{self}};

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

pub fn get_track(lr: &mut xml_reader::LibraryXmlReader) -> Result<Track, TrackErr> {
    let _ = lr.forward();
    let mut the_track = Track::default();
    loop {
        match lr.peek() {
            XmlEvent::StartElement { .. } => {
                let key = lr.element_as_string(Some("key")).unwrap();
                let value = lr.element_as_string(None).unwrap();
                match key.as_str() {
                    // skipped keys
                    "Album Rating Computed" => {}
                    "Artwork Count" => {}
                    "BPM" => {}
                    "Bit Rate" => {}
                    "Comments" => {}
                    "Disabled" => {}
                    "File Folder Count" => {}
                    "Kind" => {}
                    "Library Folder Count" => {}
                    "Movement Count" => {}
                    "Movement Name" => {}
                    "Normalization" => {}
                    "Part Of Gapless Album" => {}
                    "Play Date" => {} // use utc
                    "Rating Computed" => {}
                    "Sample Rate" => {}
                    "Size" => {}
                    "Sort Album Artist" => {}
                    "Sort Album" => {}
                    "Sort Artist" => {}
                    "Sort Composer" => {}
                    "Sort Name" => {}
                    "Track Type" => {}
                    "Volume Adjustment" => {}

                    // stored keys
                    "Album Rating" => the_track.album_rating = value.parse::<usize>()?,
                    "Release Date" => the_track.release_data = value.parse::<DateTime<Utc>>()?,
                    "Track ID" => the_track.id = value.parse::<TrackID>()?,
                    "Album Artist" => the_track.album_artist = value,
                    "Album" => the_track.album_title = value,
                    "Artist" => the_track.artist = value,
                    "Compilation" => the_track.compiltion = true,
                    "Composer" => the_track.composer = value,
                    "Date Added" => the_track.date_added = value.parse::<DateTime<Utc>>()?,
                    "Date Modified" => the_track.date_modified = value.parse::<DateTime<Utc>>()?,
                    "Disc Count" => the_track.disc_count = value.parse::<usize>()?,
                    "Disc Number" => the_track.disc_number = value.parse::<usize>()?,
                    "Genre" => the_track.genre = value,
                    "Grouping" => the_track.grouping = Some(value),
                    "Location" => {
                        the_track.location = match urlencoding::decode(&value) {
                            Err(_) => {
                                let title = &the_track.title;
                                println!("Warning: Location of \"{title}\" appears corrupt.\nThe raw value is:\n{value}");
                                "".to_owned()
                            }
                            Ok(ok) => ok.to_string(),
                        }
                    }
                    "Movement Number" => the_track.movement_number = Some(value.parse::<usize>()?),
                    "Name" => the_track.title = value,
                    "Persistent ID" => the_track.persistent_id = value,
                    "Play Count" => the_track.play_count = value.parse::<usize>()?,
                    "Play Date UTC" => the_track.play_date = Some(value.parse::<DateTime<Utc>>()?),
                    "Rating" => the_track.rating = value.parse::<usize>()?,
                    "Skip Count" => the_track.skip_count = value.parse::<usize>()?,
                    "Skip Date" => the_track.skip_date = Some(value.parse::<DateTime<Utc>>()?),
                    "Total Time" => {
                        the_track.duration = Duration::from_millis(value.parse::<u64>()?)
                    }
                    "Track Count" => the_track.total_tracks = value.parse::<usize>()?,
                    "Track Number" => the_track.track_number = value.parse::<usize>()?,
                    "Work" => the_track.work = Some(value),
                    "Year" => the_track.year = value.parse::<usize>()?,

                    _ => {
                        let title = &the_track.title;
                        let artist = &the_track.artist;
                        println!("Unexpected key:\n\t\"{key}\" with value\n\t\"{value}\"\nwhen reading{title} by{artist}.");
                    }
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "dict" {
                    let _ = lr.forward();
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

pub fn get_tracks(
    lr: &mut LibraryXmlReader,
) -> Result<TrackMap, xml_reader::err::LibraryXmlReader> {
    let mut track_map = TrackMap::new();
    // process each track
    lr.eat_start("dict")?;
    loop {
        match lr.peek() {
            XmlEvent::StartElement { name, .. } => {
                //
                if name.local_name == "key" {
                    let id = lr
                        .element_as_string(Some("key"))
                        .unwrap()
                        .parse::<usize>()
                        .expect("id?");
                    let track = get_track(lr)?;
                    assert_eq!(id, track.id);
                    track_map.insert(id, track);
                } else {
                    panic!("Failed to process track {}", lr.parser.position());
                }
            }
            XmlEvent::EndElement { .. } => {
                lr.eat_end("dict")?;
                break;
            }
            _ => {}
        }
    }
    println!("track count: {}", track_map.keys().count());
    Ok(track_map)
}
