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

use rusqlite::{Connection, Result, ToSql};

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
        let mut query_prep: Vec<(&str, &dyn ToSql)> = vec![];
        let mut query_where = vec![];

        if let Some(artist) = &track.artist {
            query_prep.push((":artist", artist));
            query_where.push("artist = :artist");
        }
        // to ensure the formatted string lives suffiently long, if used
        // the like is used as (at least sometimes) without a track apple music uses the filename while navidrome uses a path
        // as the filename is included in the path, things work out
        let like_hack = format!("%{}", &track.title.clone().unwrap_or("".to_string()));
        if let Some(_use_hack) = &track.title {
            query_prep.push((":title", &like_hack));
            query_where.push("title LIKE :title");
        }

        if let Some(album) = &track.album_title {
            query_prep.push((":album", album));
            query_where.push("album = :album");
        }

        if let Some(track_number) = &track.track_number {
            query_prep.push((":track_number", track_number));
            query_where.push("track_number = :track_number");
        }

        if let Some(disc_number) = &track.disc_number {
            query_prep.push((":disc_number", disc_number));
            query_where.push("disc_number = :disc_number");
        }

        let query_string = format!(
            "SELECT title, album, artist, track_number, disc_number FROM media_file WHERE {}",
            query_where.join(" AND ")
        );

        let mut stmt = db.prepare(&query_string).expect("questr");
        let rows = stmt
            .query_map(query_prep.as_slice(), |row| {
                let x: String = row.get(0).unwrap();
                Ok(x)
            })
            .unwrap();

        let mut row_details = vec![];
        let mut found = false;
        for row in rows {
            found = true;
            row_details.push(row.unwrap());
            // println!("{}", row.unwrap());
        }
        if row_details.len() > 1 {
            dbg!(&query_prep.iter().map(|(a, _)| a).collect::<Vec<_>>());
            dbg!(row_details);
            panic!("long");
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
