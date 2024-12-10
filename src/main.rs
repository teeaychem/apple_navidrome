use chrono::{DateTime, Datelike, Timelike, Utc};
use id3::{Frame, Tag, TagLike, Timestamp};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::common::{Position, TextPosition};

use xml::reader::{EventReader, ParserConfig2, XmlEvent};

type TrackID = usize;
type TrackMap = HashMap<TrackID, Track>;

struct LibraryReader {
    pub parser: EventReader<BufReader<File>>,
    pub next: XmlEvent,
}

#[allow(dead_code)]
#[derive(Debug)]
enum LibraryError {
    IO,
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
    MultipleTrackLibraries,
    MultiplePlaylistArrays,
}

impl From<xml::reader::Error> for LibraryError {
    fn from(error: xml::reader::Error) -> Self {
        LibraryError::Xml { error }
    }
}

impl LibraryReader {
    fn new(path: impl AsRef<Path>) -> Result<Self, LibraryError> {
        let Ok(file) = File::open(path) else {
            return Err(LibraryError::IO);
        };
        let reader = BufReader::new(file);
        let mut parser = ParserConfig2::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .coalesce_characters(true)
            .create_reader(reader);
        let next = parser.next()?;
        Ok(LibraryReader { parser, next })
    }

    fn peek(&self) -> &XmlEvent {
        &self.next
    }

    fn next(&mut self) -> Result<&XmlEvent, xml::reader::Error> {
        self.next = self.parser.next()?;
        Ok(&self.next)
    }

    fn eat_start(&mut self, name: &str) -> Result<(), LibraryError> {
        let XmlEvent::StartElement { name: found, .. } = self.peek() else {
            return Err(LibraryError::UnableToEat {
                position: self.parser.position(),
                want: name.to_string(),
            });
        };
        if found.local_name != name {
            return Err(LibraryError::BadEat {
                position: self.parser.position(),
                want: found.to_string(),
                got: name.to_string(),
            });
        }
        self.next = self.parser.next()?;
        Ok(())
    }

    fn eat_end(&mut self, name: &str) -> Result<(), LibraryError> {
        let XmlEvent::EndElement { name: found, .. } = self.peek() else {
            return Err(LibraryError::UnableToEat {
                position: self.parser.position(),
                want: name.to_string(),
            });
        };
        if found.local_name != name {
            return Err(LibraryError::BadEat {
                position: self.parser.position(),
                want: found.to_string(),
                got: name.to_string(),
            });
        }
        self.next = self.parser.next()?;
        Ok(())
    }

    fn element_as_string(&mut self, name: Option<&str>) -> Result<String, LibraryError> {
        let XmlEvent::StartElement { name: start, .. } = self.peek() else {
            return Err(LibraryError::KeyNotStart {
                position: self.parser.position(),
            });
        };
        let element = start.local_name.clone();
        if let Some(name_check) = name {
            if start.local_name != name_check {
                return Err(LibraryError::BadEat {
                    position: self.parser.position(),
                    want: name_check.to_owned(),
                    got: start.local_name.to_owned(),
                });
            }
        }
        let as_string = match element.as_str() {
            "true" => {
                let _ = self.next();
                element.clone()
            }
            "false" => {
                let _ = self.next();
                element.clone()
            }
            "string" => {
                //
                self.next()?;
                match self.peek() {
                    XmlEvent::Characters(chars) => {
                        let the_string = chars.to_owned();
                        self.next()?;
                        the_string
                    }
                    _ => "".to_owned(),
                }
            }
            _ => {
                let XmlEvent::Characters(chars) = self.next()? else {
                    return Err(LibraryError::UnexpectedKey {
                        position: self.parser.position(),
                        key: element,
                    });
                };

                let the_string = chars.to_owned();
                self.next()?;
                the_string
            }
        };

        let XmlEvent::EndElement { name: end, .. } = self.peek() else {
            return Err(LibraryError::UnexpectedElement {
                position: self.parser.position(),
            });
        };
        if end.local_name != element {
            return Err(LibraryError::BadPair {
                position: self.parser.position(),
                start: element,
                end: end.local_name.clone(),
            });
        }
        if let Some(name_check) = name {
            if end.local_name != name_check {
                return Err(LibraryError::BadEat {
                    position: self.parser.position(),
                    want: name_check.to_owned(),
                    got: end.local_name.to_owned(),
                });
            }
        }
        self.next()?;

        Ok(as_string)
    }
}

#[derive(Debug, Default)]
struct Track {
    id: TrackID,
    persistent_id: String,

    tag: Tag,

    grouping: Option<String>,
    work: Option<String>,
    movement_number: Option<usize>,

    date_modified: DateTime<Utc>,
    date_added: DateTime<Utc>,

    play_count: usize,
    play_date: Option<DateTime<Utc>>,

    skip_count: usize,
    skip_date: Option<DateTime<Utc>>,

    rating: usize,
    album_rating: usize,

    compiltion: bool,

    location: String,
}

fn get_track(lr: &mut LibraryReader) -> Track {
    let _ = lr.next();
    let mut the_track = Track::default();
    loop {
        match lr.peek() {
            XmlEvent::StartElement { .. } => {
                let key = lr.element_as_string(Some("key")).unwrap();
                let value = lr.element_as_string(None).unwrap();
                match key.as_str() {
                    "Album Rating Computed"
                    | "Artwork Count"
                    | "Bit Rate"
                    | "Disabled"
                    | "Disc Count"
                    | "File Folder Count"
                    | "Kind"
                    | "Library Folder Count"
                    | "Movement Count"
                    | "Movement Name"
                    | "Normalization"
                    | "Part Of Gapless Album"
                    | "Play Date" // use utc
                    | "Rating Computed"
                    | "Sample Rate"
                    | "Size"
                    | "Sort Album Artist"
                    | "Sort Album"
                    | "Sort Artist"
                    | "Sort Composer"
                    | "Sort Name"
                    | "Total Time"
                    | "Track Type" => {
                        // no corresponding id3v2
                    }
                    // id3v2 tags, with given id3 setters
                    "Album Artist" => the_track.tag.set_album_artist(value.to_owned()),
                    "Album" => the_track.tag.set_album(value.to_owned()),
                    "Artist" => the_track.tag.set_artist(value.to_owned()),
                    "Disc Number" => the_track.tag.set_disc(value.parse::<u32>().expect("disc number?")),
                    "Genre" => the_track.tag.set_genre(value.to_owned()),
                    "Name" => the_track.tag.set_title(value.to_owned()),
                    "Track Count" => the_track.tag.set_total_tracks(value.parse::<u32>().expect("track count?")),
                    "Track Number" => the_track.tag.set_track(value.parse::<u32>().expect("track number?")),
                    "Year" => the_track.tag.set_year(value.parse::<i32>().expect("year?")),

                    // id3v2 tags, with (specification defined) custom id3 setters
                    "BPM" => {
                        the_track.tag.add_frame(Frame::text("BPM", value));
                    }
                    "Comments" => {
                        the_track.tag.add_frame(id3::frame::Comment {
                            lang: "".to_string(),
                            description: "".to_string(),
                            text: value.to_owned(),
                        });
                    }
                    "Composer" => {
                        the_track.tag.add_frame(Frame::text("TCOM", value));
                    }
                    "Release Date" => {
                        let utc = value.parse::<DateTime<Utc>>().expect("release date?");
                        let timestamp = Timestamp {
                            year: utc.year(),
                            month: Some(utc.month() as u8),
                            day: Some(utc.day() as u8),
                            hour: Some(utc.hour() as u8),
                            minute: Some(utc.minute() as u8),
                            second: Some(utc.second() as u8),
                        };
                        the_track.tag.set_date_released(timestamp)
                    }
                    "Volume Adjustment" => {
                        the_track.tag.add_frame(Frame::text("RVAD", value));
                    }

                    // non-ID3v2 things
                    "Track ID" => the_track.id = value.parse::<TrackID>().expect("id?"),
                    "Persistent ID" => the_track.persistent_id = value.to_owned(),

                    "Grouping" => the_track.grouping = Some(value.to_owned()),
                    "Work" => the_track.work = Some(value.to_owned()),
                    "Movement Number" => {
                        the_track.movement_number =
                            Some(value.parse::<usize>().expect("mvn?"))
                    }

                    "Date Modified" => {
                        the_track.date_modified = value.parse::<DateTime<Utc>>().expect("dm?")
                    }
                    "Date Added" => {
                        the_track.date_added = value.parse::<DateTime<Utc>>().expect("da?")
                    }
                    "Location" => {
                        the_track.location = match urlencoding::decode(&value) {
                            Err(_) => {
                                let the_title = the_track.tag.title().unwrap_or("some track");
                                println!("Warning: The location of \"{the_title}\" in the library appears corrupt.
The raw value is:
{value}",);
                                "".to_owned()
                            },
                            Ok(ok) => ok.to_string()

}
                    }
                    "Play Count" => {
                        the_track.play_count = value.parse::<usize>().expect("play count?");
                        the_track.tag.add_frame(Frame::text("PCNT", value));
                    }

                    "Play Date UTC" => {
                        the_track.play_date =
                            Some(value.parse::<DateTime<Utc>>().expect("play date?"))
                    }

                    "Skip Count" => {
                        the_track.skip_count = value.parse::<usize>().expect("skip count?")
                    }

                    "Skip Date" => {
                        the_track.skip_date =
                            Some(value.parse::<DateTime<Utc>>().expect("skip date?"))
                    }

                    "Rating" => the_track.rating = value.parse::<usize>().expect("rating?"),

                    "Album Rating" => {
                        the_track.album_rating =
                            value.parse::<usize>().expect("album rating?")
                    }

                    "Compilation" => the_track.compiltion = true,
                    _ => {
                        panic!("{key} - {value}");
                    }
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "dict" {
                    let _ = lr.next();
                    break;
                } else {
                    panic!();
                }
            }
            _ => {}
        }
    }
    the_track
}

fn get_tracks(lr: &mut LibraryReader) -> Result<TrackMap, LibraryError> {
    let mut track_map = HashMap::<usize, Track>::new();
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
                    let track = get_track(lr);
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

fn playlist_ids(lr: &mut LibraryReader) -> Result<Vec<usize>, LibraryError> {
    let mut ids = Vec::default();
    lr.eat_start("array")?;
    loop {
        match lr.peek() {
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == "dict" {
                    lr.eat_start("dict")?;
                    let _key = lr.element_as_string(Some("key"));
                    let id = lr
                        .element_as_string(Some("integer"))
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    ids.push(id);
                    lr.eat_end("dict")?;
                }
            }
            XmlEvent::EndElement { .. } => {
                lr.eat_end("array")?;
                break;
            }
            _ => {}
        }
    }
    Ok(ids)
}

#[derive(Default, Debug)]
struct Playlist {
    name: String,
    description: String,
    persistent_id: String,
    parent_persistent_id: String,
    folder: bool,
    items: Vec<usize>,
}

fn get_playlist(lr: &mut LibraryReader) -> Result<Playlist, LibraryError> {
    let mut the_playlist = Playlist::default();
    lr.eat_start("dict")?;
    loop {
        match lr.peek() {
            XmlEvent::StartElement { name, .. } => {
                //
                match name.local_name.as_str() {
                    "key" => {
                        let key = lr.element_as_string(Some("key")).unwrap();
                        if key == "Playlist Items" {
                            the_playlist.items = playlist_ids(lr)?;
                        } else {
                            let value = lr.element_as_string(None).unwrap();
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
                                    lr.parser.position()
                                ),
                            }
                        }
                    }
                    _ => panic!("Failed to process track {}", lr.parser.position()),
                }
            }
            XmlEvent::EndElement { .. } => {
                lr.eat_end("dict")?;
                break;
            }
            _ => {}
        }
    }
    Ok(the_playlist)
}

fn get_playlists(lr: &mut LibraryReader) -> Result<Vec<Playlist>, LibraryError> {
    let mut the_lists = Vec::default();
    lr.eat_start("array")?;
    // process each track
    loop {
        match lr.peek() {
            XmlEvent::StartElement { name, .. } => {
                //
                match name.local_name.as_str() {
                    "dict" => {
                        the_lists.push(get_playlist(lr)?);
                    }
                    _ => panic!("Failed to process track {}", lr.parser.position()),
                }
            }
            XmlEvent::EndElement { .. } => {
                //
                lr.eat_end("array")?;
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

fn tracks_and_playlists(
    path: &str,
) -> Result<(Option<TrackMap>, Option<Vec<Playlist>>), LibraryError> {
    let mut lr = LibraryReader::new(path).unwrap();
    let mut track_map = None;
    let mut playlists = None;
    // skip until library dictionary
    loop {
        if let Ok(XmlEvent::StartElement { name, .. }) = lr.next() {
            if name.local_name == "dict" {
                break;
            }
        }
    }
    lr.eat_start("dict")?;
    loop {
        match lr.peek() {
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == "key" {
                    let key = lr.element_as_string(Some("key")).unwrap();
                    match key.as_str() {
                        "Tracks" => {
                            if track_map.is_some() {
                                return Err(LibraryError::MultipleTrackLibraries);
                            }
                            track_map = Some(get_tracks(&mut lr)?);
                        }
                        "Playlists" => {
                            if playlists.is_some() {
                                return Err(LibraryError::MultiplePlaylistArrays);
                            }
                            playlists = Some(get_playlists(&mut lr)?);
                        }
                        _ => {
                            print!("{key} : ");
                            let value = lr.element_as_string(None).unwrap();
                            println!("{value}");
                        }
                    }
                } else {
                    panic!(
                        "{} :Unexpected xml start element {name}",
                        lr.parser.position()
                    );
                }
            }
            XmlEvent::EndElement { .. } => {
                lr.eat_end("dict")?;
                break;
            }
            _ => {
                panic!(
                    "{} : Unexpected xml event {:?}",
                    lr.parser.position(),
                    lr.peek()
                );
            }
        }
    }
    Ok((track_map, playlists))
}

fn main() -> Result<(), LibraryError> {
    let (tracks_mb, playlists_mb) = match tracks_and_playlists("Library.xml") {
        Ok(yes) => yes,
        Err(no) => panic!("{no:?}"),
    };
    if let Some(tracks) = &tracks_mb {
        println!("Read {} tracks", tracks.keys().count());
    }
    if let Some(playlists) = &playlists_mb {
        println!("Read {} playlists", playlists.len());
    }
    let skip_lists = HashSet::from(["Library", "Downloaded", "Music"]);

    if let (Some(tracks), Some(playlists)) = (tracks_mb, playlists_mb) {
        for playlist in playlists {
            println!("{}", playlist.name);
            if skip_lists.contains(playlist.name.as_str()) || playlist.folder {
                continue;
            }
            for id in &playlist.items {
                let track = tracks.get(id).expect("missing track id");
                println!("  {} ({}) ", track.tag.title().unwrap(), track.play_count);
                println!("  - {}", track.location);
            }
        }
    }

    Ok(())
}
