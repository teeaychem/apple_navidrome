use chrono::{DateTime, Utc};
use std::time::Duration;

use xml::{common::Position, reader::XmlEvent};

use crate::{
    structs::{track::Track, Library},
    xml_reader::{
        self,
        err::{self},
    },
};

use super::LibraryXmlReader;

/// Reads a track from `reader` and returns a [Track], if possible.
///
/// Coverage for keys is incomplete, and splits into three cases:
/// - A known key has some corresponding [Track] field, in which case the key is recorded to the track.
/// - A known key has no relevant Track field, in which case the key is ignored.
/// - An unknown key is found, in which case a debug message is written and the key is ignored.
///
/// For example, "Play Date" is a known key but is ignored as the known key "Play Date UTC" allows for easier parsing.
///
/// There is no difference in effect between an known key with no relevant Track field and an unknown key, with the exception of a log.
/// Though, if you'd like to extend [Track] (etc.) then perhaps the known keys are useful!
pub fn get_track(
    reader: &mut xml_reader::LibraryXmlReader,
) -> Result<Track, err::LibraryXmlReader> {
    let _ = reader.forward();
    let mut track = Track::default();
    loop {
        match reader.peek() {
            XmlEvent::StartElement { .. } => {
                let key = reader.element_as_string(Some("key"))?;
                let value = reader.element_as_string(None)?;
                match key.as_str() {
                    "Album Artist" => track.album_artist = Some(value),
                    "Album Rating Computed" => {}
                    "Album Rating" => track.album_rating = Some(value.parse::<usize>()?),
                    "Album" => track.album_title = Some(value),
                    "Artist" => track.artist = Some(value),
                    "Artwork Count" => {}
                    "BPM" => track.bpm = Some(value.parse::<usize>()?),
                    "Bit Rate" => {}
                    "Clean" => {}
                    "Comments" => track.comments = Some(value),
                    "Compilation" => track.compiltion = true,
                    "Composer" => track.composer = Some(value),
                    "Date Added" => track.date_added = value.parse::<DateTime<Utc>>()?,
                    "Date Modified" => track.date_modified = value.parse::<DateTime<Utc>>()?,
                    "Explicit" => {}
                    "Disc Count" => track.disc_count = Some(value.parse::<usize>()?),
                    "Disc Number" => track.disc_number = Some(value.parse::<usize>()?),
                    "Disabled" => {}
                    "Disliked" => {}
                    "Favorited" => track.favourited = true,
                    "File Folder Count" => {}
                    "Genre" => track.genre = Some(value),
                    "Grouping" => track.grouping = Some(value),
                    "Kind" => {}
                    "Library Folder Count" => {}
                    "Location" => track.location = value,
                    "Loved" => track.loved = true,
                    "Movement Count" => {}
                    "Movement Name" => track.movement_title = Some(value),
                    "Movement Number" => track.movement_number = Some(value.parse::<usize>()?),
                    "Name" => track.title = Some(value),
                    "Normalization" => {}
                    "Part Of Gapless Album" => {}
                    "Persistent ID" => track.persistent_id = value,
                    "Play Count" => track.play_count = value.parse::<usize>()?,
                    "Play Date UTC" => track.play_date = Some(value.parse::<DateTime<Utc>>()?),
                    "Play Date" => {} // use utc variant
                    "Playlist Only" => {}
                    "Purchased" => {}
                    "Rating Computed" => {}
                    "Rating" => track.rating = value.parse::<usize>()?,
                    "Release Date" => track.release_data = Some(value.parse::<DateTime<Utc>>()?),
                    "Sample Rate" => {}
                    "Size" => track.size = value.parse::<usize>()?,
                    "Skip Count" => track.skip_count = value.parse::<usize>()?,
                    "Skip Date" => track.skip_date = Some(value.parse::<DateTime<Utc>>()?),
                    "Sort Album Artist" => {}
                    "Sort Album" => {}
                    "Sort Artist" => {}
                    "Sort Composer" => {}
                    "Sort Name" => {}
                    "Sort Series" => {}
                    "Total Time" => track.duration = Duration::from_millis(value.parse::<u64>()?),
                    "Track Count" => track.total_tracks = Some(value.parse::<usize>()?),
                    "Track ID" => track.id = value,
                    "Track Number" => track.track_number = Some(value.parse::<usize>()?),
                    "Track Type" => {}
                    "Volume Adjustment" => {}
                    "Work" => track.work = Some(value),
                    "Year" => track.year = Some(value.parse::<usize>()?),
                    // For any unknown key, log and skip.
                    _ => {
                        log::debug!(
                            "Ignoring track key '{key}' with value '{value}' in track '{}' by '{}'",
                            track.title.as_deref().unwrap_or("[No title]"),
                            track.artist.as_deref().unwrap_or("[No artist]")
                        );
                        continue;
                    }
                }
            }
            XmlEvent::EndElement { name } => match name.local_name.as_str() {
                "dict" => {
                    let _ = reader.forward();
                    break;
                }
                _ => {
                    return Err(err::LibraryXmlReader::UnexpectedKey {
                        position: reader.parser.position(),
                        key: name.local_name.to_owned(),
                    })
                }
            },
            _ => {
                log::debug!(
                    "Ignoring unexpected element '{:?}' in track dict",
                    reader.peek()
                );
                let _ = reader.forward();
                continue;
            }
        }
    }
    Ok(track)
}

impl Library {
    pub fn import_tracks(
        &mut self,
        reader: &mut LibraryXmlReader,
    ) -> Result<(), xml_reader::err::LibraryXmlReader> {
        // Process each track in the tracks dictionary.
        reader.eat_start("dict")?;
        loop {
            match reader.peek() {
                XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                    "key" => {
                        let id = reader.element_as_string(Some("key"))?;
                        let track = get_track(reader)?;
                        assert_eq!(id, track.id);
                        self.tracks.insert(id, track);
                    }
                    _ => {
                        return Err(err::LibraryXmlReader::UnexpectedKey {
                            position: reader.parser.position(),
                            key: name.local_name.to_owned(),
                        });
                    }
                },
                XmlEvent::EndElement { .. } => {
                    reader.eat_end("dict")?;
                    break;
                }
                _ => {
                    log::debug!(
                        "Ignoring unexpected element '{:?}' in track array",
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
