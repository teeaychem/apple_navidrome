use std::path::Path;

use apple_navidrome_lib::xml_reader::{self, *};

const PLAYLIST_DIR: &str = "./playlists";

use rusqlite::{Connection, Result};

fn main() -> Result<(), xml_reader::err::LibraryXmlReader> {
    // let nd_db_path = "./navidrome.db";
    // let db = Connection::open(nd_db_path).unwrap();
    // println!("{}", db.is_autocommit());
    // let mut stmt = db.prepare("SELECT title, album, artist, track_number FROM media_file").unwrap();
    // let person_iter = stmt
    //     .query_map([], |row| {
    //         let title: String = row.get(0).unwrap();
    //         let album: String = row.get(1).unwrap();
    //         let artist: String = row.get(2).unwrap();
    //         let track: usize = row.get(3).unwrap();
    //         Ok(format!("{track} - {title}\n{artist}\n{album}"))
    //     })
    //     .unwrap();
    // for person in person_iter {
    //     println!("{}", person.unwrap());
    // }

    let library = build_library(Path::new("Library.xml")).unwrap();

    let list_as_json = serde_json::to_string(&library).unwrap();

    let mut file = std::fs::File::create("Library.json").expect("Could not create file!");

    std::io::Write::write_all(&mut file, list_as_json.as_bytes())
        .expect("Cannot write to the file!");

    println!("Read {} tracks", library.tracks.keys().count());
    println!("Read {} playlists", library.playlists.len());
    let skip_lists = Vec::from(["Library", "Downloaded", "Music"]);

    let playlist_dir_path = std::path::Path::new(PLAYLIST_DIR);
    std::fs::create_dir(playlist_dir_path);
    for playlist in &library.playlists {
        if skip_lists.iter().any(|l| *l == playlist.name) || playlist.folder {
            continue;
        }
        println!("Creating {}", playlist.name);
        playlist.export_m3u8(playlist_dir_path, &library.tracks);
    }

    Ok(())
}
