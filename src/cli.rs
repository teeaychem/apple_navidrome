use apple_navidrome_lib::{
    navidrome_writer::{NavidromeWriter, TrackMatcher},
    structs::{track::Track, Library},
};

/*
Notes on fields:

- size is not consistent between navidrome and apple music
- navidrome does not consistenly assign a track number if a number is not given (both 0 and 1 observed)
- things break if ';' is in an artist, both for queries and apple music
 */

pub mod err {
    use apple_navidrome_lib::xml_reader;

    #[derive(Debug)]
    pub enum Cli {
        LibraryXmlReader(xml_reader::err::LibraryXmlReader),
        NavidromeSql(rusqlite::Error),
        Json(serde_json::Error),
        Io(std::io::Error),
    }

    impl From<std::io::Error> for Cli {
        fn from(error: std::io::Error) -> Self {
            Cli::Io(error)
        }
    }

    impl From<serde_json::Error> for Cli {
        fn from(error: serde_json::Error) -> Self {
            Cli::Json(error)
        }
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
    let mut clog = colog::default_builder();
    clog.filter_level(log::LevelFilter::Debug);
    clog.init();

    let mut library = Library::from_xml(std::path::Path::new("Library.xml"))?;
    // let library = Library::from_json(std::path::Path::new("Library.json")).unwrap();
    log::info!("Read {} tracks", library.tracks.keys().count());
    log::info!("Read {} playlists", library.playlists.len());
    library.artist_album_playcounts();

    let nd_db_path = "./navidrome.db";
    let backup = format!("{nd_db_path}.backup.db");
    match std::fs::copy(nd_db_path, &backup) {
        Err(_) => {
            log::error!(
                "Failed to create a copy of the navidrome database, no further action taken"
            );
            std::process::exit(1)
        }
        Ok(_) => {
            log::info!("A copy of the navidrome database has made.");
        }
    };
    let writer = NavidromeWriter::from(std::path::Path::new(&backup))?;
    let user_id = {
        let user = "user";
        let ids = writer.user_ids(user)?;
        match &ids[..] {
            [] => {
                log::error!("No user \"{user}\" found.");
                std::process::exit(1);
            }
            [unique] => {
                log::info!("User \"{user}\" found with id: {unique}");
                unique.to_owned()
            }
            _ => {
                log::error!("Multiple ids found for user \"{user}\".");
                std::process::exit(1);
            }
        }
    };

    let mut failed_matches = vec![];
    let mut multiple_matches = vec![];
    for track in library.tracks.values() {
        let mut matcher = TrackMatcher::from_track(track);
        let ids = writer.item_ids(&mut matcher)?;
        match ids.len() {
            0 => failed_matches.push(track), // missing track
            1 => {
                // unique track
                writer.update_match(&matcher, &user_id);
                // update_match(&matcher, &db)?;
            }
            _ => multiple_matches.push(track), // multiple tracks
        }
    }
    if !failed_matches.is_empty() {
        match write_failed_matches(failed_matches) {
            Ok(_) => {},
            Err(e) => log::warn!("Some tracks from Apple Music could not be matched to a track in the navidrome database.
An error occurred when attempting to write these to a file: {e:?}")
}
    }
    if !multiple_matches.is_empty() {
        match write_multiple_matches(multiple_matches) {
            Ok(_) => {},
            Err(e) => log::warn!("Some tracks from Apple Music were matched to multiple tracks in the navidrome database.
An error occurred when attempting to write these to a file: {e:?}")
}
    }

    match writer.update_artist_album_counts(&library, &user_id) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error updating artist counts:\n{e:?}");
        }
    };

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

pub fn write_failed_matches(failed_matches: Vec<&Track>) -> Result<(), err::Cli> {
    let fail_match_path = "failed_matches.json";
    let mut fail_match_file = std::fs::File::create(fail_match_path)?;
    let _ = std::io::Write::write_all(
        &mut fail_match_file,
        serde_json::to_string_pretty(&failed_matches)?.as_bytes(),
    );
    log::warn!(
        "Some tracks from Apple Music could not be matched to a track in the navidrome database.
A JSON file containing these tracks has been written to {fail_match_path}."
    );
    Ok(())
}

pub fn write_multiple_matches(multiple_matches: Vec<&Track>) -> Result<(), err::Cli> {
    let mismatch_path = "multiple_matches.json";
    let mut mismatch_file = std::fs::File::create(mismatch_path)?;

    let _ = std::io::Write::write_all(
        &mut mismatch_file,
        serde_json::to_string_pretty(&multiple_matches)?.as_bytes(),
    );
    log::warn!(
        "Some tracks from Apple Music were matched to multiple tracks in the navidrome database.
A JSON file containing these tracks has been written to {mismatch_path}."
    );
    Ok(())
}
