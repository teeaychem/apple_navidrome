use apple_navidrome_lib::{
    structs::Library,
    xml_reader::{self},
};

/*
Notes on fields:

- size is not consistent between navidrome and apple music
- navidrome does not consistenly assign a track number if a number is not given (both 0 and 1 observed)
- things break if ';' is in an artist, both for queries and apple music
 */

const PLAYLIST_DIR: &str = "./playlists";

use rusqlite::{Connection, Result};

fn main() -> Result<(), xml_reader::err::LibraryXmlReader> {
    let library = Library::from_xml(std::path::Path::new("Library.xml")).unwrap();
    // let library = Library::from_json(std::path::Path::new("Library.json")).unwrap();
    println!("Read {} tracks", library.tracks.keys().count());
    println!("Read {} playlists", library.playlists.len());
    let mut not_found = vec![];

    let nd_db_path = "./navidrome.db";
    let db = Connection::open(nd_db_path).unwrap();
    println!("{}", db.is_autocommit());

    for track in library.tracks.values() {
        let artist = match &track.artist {
            None => "[Unknown Artist]",
            Some(artist) => artist,
        };
        let title = match &track.title {
            None => "",
            Some(title) => title,
        };
        let album = match &track.album_title {
            None => "",
            Some(album) => album,
        };

        let mut query_string: String =
            "SELECT title, album, artist, track_number, size FROM media_file WHERE ".to_string();
        let constraints = [
            "title LIKE :title",
            "album = :album",
            "artist = :artist",
            // "track_number = :track_number",
        ];
        query_string.push_str(&constraints.join(" AND "));

        let mut stmt = db.prepare(&query_string).unwrap();
        let rows = stmt
            .query_map(
                &[
                    (":title", format!("%{title}%").as_str()),
                    (":album", album),
                    (":artist", artist),
                    // (
                    //     ":track_number",
                    //     &track.track_number.unwrap_or(1).to_string(),
                    // ),
                ],
                |row| {
                    let x: String = row.get(0).unwrap();
                    Ok(x)
                },
            )
            .unwrap();

        let mut found = false;
        for row in rows {
            found = true;
            // println!("{}", row.unwrap());
        }
        if !found {
            not_found.push(track);
        }
    }

    for nf in &not_found {
        dbg!(nf);
    }

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
