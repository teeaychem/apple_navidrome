use apple_navidrome_lib::structs::{track::Track, Library};

/*
Notes on fields:

- size is not consistent between navidrome and apple music
- navidrome does not consistenly assign a track number if a number is not given (both 0 and 1 observed)
- things break if ';' is in an artist, both for queries and apple music
 */

const PLAYLIST_DIR: &str = "./playlists";

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, ToSql};

pub mod err {
    use apple_navidrome_lib::xml_reader;

    #[derive(Debug)]
    pub enum Cli {
        LibraryXmlReader(xml_reader::err::LibraryXmlReader),
        NavidromeSql(rusqlite::Error),
    }

    impl From<xml_reader::err::LibraryXmlReader> for Cli {
        fn from(error: xml_reader::err::LibraryXmlReader) -> Self {
            Cli::LibraryXmlReader(error)
        }
    }

    impl From<rusqlite::Error> for Cli {
        fn from(error: rusqlite::Error) -> Self {
            Cli::NavidromeSql(error)
        }
    }
}

fn main() -> Result<(), err::Cli> {
    let library = Library::from_xml(std::path::Path::new("Library.xml"))?;
    // let library = Library::from_json(std::path::Path::new("Library.json")).unwrap();
    println!("Read {} tracks", library.tracks.keys().count());
    println!("Read {} playlists", library.playlists.len());

    let nd_db_path = "./navidrome.db";
    let db = Connection::open(nd_db_path)?;

    let user = "user";
    let user_id = {
        let ids = user_ids(user, &db)?;
        match &ids.len() {
            0 => panic!("No user \"{user}\" found."),
            1 => ids.first().unwrap().to_owned(),
            _ => {
                dbg!(ids);
                panic!("Multiple ids found for user \"{user}\"")
            }
        }
    };
    println!("User id is: {user_id}");

    for track in library.tracks.values() {
        let mut matcher = TrackMatcher::from_track(track);
        let ids = item_ids(&mut matcher, &db)?;
        match ids.len() {
            0 => {
                println!("Failed to match track");
                dbg!(track);
                dbg!(matcher.selections);
                // missing track
            }
            1 => {
                // unique track
                println!("{:?} -> {}", &track.title, ids.first().unwrap());
                update_match(&matcher, &user_id, &db);
                // update_match(&matcher, &db)?;
            }
            _ => {
                // multiple tracks
            }
        }
    }

    // for nf in &not_found {
    //     dbg!(nf);
    // }

    // println!("Read {} tracks", library.tracks.keys().count());
    // println!("Read {} playlists", library.playlists.len());
    // let skip_lists = Vec::from(["Library", "Downloaded", "Music"]);

    // let playlist_dir_path = std::path::Path::new(PLAYLIST_DIR);
    // std::fs::create_dir(playlist_dir_path);
    // for playlist in &library.playlists {
    //     if skip_lists.iter().any(|l| *l == playlist.name) || playlist.folder {
    //         continue;
    //     }
    //     println!("Creating {}", playlist.name);
    //     playlist.export_m3u(playlist_dir_path, &library.tracks);
    // }

    Ok(())
}

pub struct TrackMatcher<'t> {
    track: &'t Track,
    selections: Vec<&'t str>,
    binds: Vec<&'t str>,
    parameters: Vec<(&'t str, &'t dyn ToSql)>,
    item_id: Option<String>,
}

impl<'t> TrackMatcher<'t> {
    fn from_track(track: &'t Track) -> Self {
        let mut identifier = TrackMatcher {
            track,
            selections: vec![],
            binds: vec![],
            parameters: vec![],
            item_id: None,
        };

        if let Some(artist) = &track.artist {
            identifier.selections.push("artist");
            identifier.parameters.push((":artist", artist));
            identifier.binds.push("artist = :artist");
        }

        if let Some(album) = &track.album_title {
            identifier.selections.push("album");
            identifier.parameters.push((":album", album));
            identifier.binds.push("album = :album");
        }

        // to ensure the formatted string lives suffiently long, if used
        // the like is used as (at least sometimes) without a track apple music uses the filename while navidrome uses a path
        // as the filename is included in the path, things work out
        let track_hack = Box::leak(Box::new(format!(
            "%{}",
            &track.title.clone().unwrap_or("".to_string())
        )));
        if let Some(_use_hack) = &track.title {
            identifier.selections.push("title");
            identifier.parameters.push((":title", track_hack));
            identifier.binds.push("title LIKE :title");
        }

        if let Some(track_number) = &track.track_number {
            identifier.selections.push("track_number");
            identifier.parameters.push((":track_number", track_number));
            identifier.binds.push("track_number = :track_number");
        }

        if let Some(disc_number) = &track.disc_number {
            identifier.selections.push("disc_number");
            identifier.parameters.push((":disc_number", disc_number));
            identifier.binds.push("disc_number = :disc_number");
        }

        identifier
    }

    pub fn selects(&self) -> String {
        self.selections.join(", ")
    }

    pub fn binds(&self) -> String {
        self.binds.join(" AND ")
    }

    pub fn parameters(&self) -> &[(&'t str, &'t dyn ToSql)] {
        self.parameters.as_slice()
    }
}

pub fn item_ids(
    matcher: &mut TrackMatcher,
    db: &Connection,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut item_ids: Vec<String> = vec![];

    let query_string = format!(
        "SELECT id, {} FROM media_file WHERE {}",
        matcher.selects(),
        matcher.binds()
    );

    let mut stmt = db.prepare(&query_string)?;
    let mut rows = stmt.query(matcher.parameters())?;
    while let Some(row) = rows.next()? {
        let id: Option<String> = row.get("id").unwrap();
        if let Some(found) = id {
            item_ids.push(found.clone());
            matcher.item_id = Some(found);
        }
    }

    Ok(item_ids)
}

pub fn update_match(
    matcher: &TrackMatcher,
    user_id: &str,
    db: &Connection,
) -> Result<(), rusqlite::Error> {
    let update_string = "
INSERT OR REPLACE INTO
annotation
(user_id, item_id, item_type, play_count, play_date, rating, starred, starred_at)
VALUES
(
:user_id,
:item_id,
:item_type,
:play_count,
:play_date,
:rating,
:starred,
:starred_at
)
";
    let params: [(&str, &dyn ToSql); 8] = [
        (":user_id", &user_id),
        (":item_id", &matcher.item_id),
        (":item_type", &"media_file"),
        (":play_count", &matcher.track.play_count),
        (":play_date", &matcher.track.play_date),
        (":rating", &matcher.track.rating),
        (
            ":starred",
            &(matcher.track.loved || matcher.track.favourited),
        ),
        (":starred_at", &None::<DateTime<Utc>>),
    ];

    let mut stmt = db.prepare(update_string).expect("oh");
    match stmt.execute(&params) {
        Err(e) => {
            println!("Error executing update: {e:?}");
            Ok(())
        }
        Ok(_) => Ok(()),
    }
}

pub fn user_ids(user: &str, db: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut ids = vec![];

    let query_string = format!("SELECT id, user_name FROM user WHERE user_name = '{user}'");
    let mut stmt = db.prepare(&query_string)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        dbg!(row);
        let id = row.get("id").unwrap();
        ids.push(id);
    }

    Ok(ids)
}
