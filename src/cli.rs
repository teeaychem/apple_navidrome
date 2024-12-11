use apple_navidrome_lib::{structs::TrackMap, xml_reader::{self, *}};
use playlists::Playlist;
use tracks::get_tracks;

const PLAYLIST_DIR: &str = "./playlists";


fn tracks_and_playlists(
    path: &str,
) -> Result<(Option<TrackMap>, Option<Vec<Playlist>>), xml_reader::err::LibraryXmlReader> {
    let mut lr = LibraryXmlReader::new(path).unwrap();
    let mut track_map = None;
    let mut playlists = None;
    // skip until library dictionary
    loop {
        if let Ok(xml::reader::XmlEvent::StartElement { name, .. }) = lr.forward() {
            if name.local_name == "dict" {
                break;
            }
        }
    }
    lr.eat_start("dict")?;
    loop {
        match lr.peek() {
            xml::reader::XmlEvent::StartElement { name, .. } => {
                if name.local_name == "key" {
                    let key = lr.element_as_string(Some("key")).unwrap();
                    match key.as_str() {
                        "Tracks" => {
                            if track_map.is_some() {
                                return Err(xml_reader::err::LibraryXmlReader::MultipleTrackLibraries);
                            }
                            track_map = Some(get_tracks(&mut lr)?);
                        }
                        "Playlists" => {
                            if playlists.is_some() {
                                return Err(xml_reader::err::LibraryXmlReader::MultiplePlaylistArrays);
                            }
                            playlists = Some(playlists::get_playlists(&mut lr)?);
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
                        xml::common::Position::position(&lr.parser)
                    );
                }
            }
            xml::reader::XmlEvent::EndElement { .. } => {
                lr.eat_end("dict")?;
                break;
            }
            _ => {
                panic!(
                    "{} : Unexpected xml event {:?}",
                    xml::common::Position::position(&lr.parser),
                    lr.peek()
                );
            }
        }
    }
    Ok((track_map, playlists))
}

fn main() -> Result<(), xml_reader::err::LibraryXmlReader> {
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
    let skip_lists = Vec::from(["Library", "Downloaded", "Music"]);

    if let (Some(tracks), Some(playlists)) = (tracks_mb, playlists_mb) {
        let playlist_dir_path = std::path::Path::new(PLAYLIST_DIR);
        std::fs::create_dir(playlist_dir_path);
        for playlist in playlists {
            if skip_lists.iter().any(|l| *l == playlist.name) || playlist.folder {
                continue;
            }
            println!("Creating {}", playlist.name);
            playlist.export_m3u8(playlist_dir_path, &tracks);
        }
    }

    Ok(())
}
