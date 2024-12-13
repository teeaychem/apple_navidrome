use apple_navidrome_lib::{config::Config, navidrome_writer::NavidromeWriter, structs::Library};

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
    clog.init();
    clog.filter_level(log::LevelFilter::Error);
    let config = Config::from_file();

    clog.filter_level(config.get_log_level());

    let mut library = Library::from_xml(&config.apple_music_library)?;
    // let library = Library::from_json(std::path::Path::new("Library.json")).unwrap();
    log::info!("Found {} tracks", library.tracks.keys().count());
    log::info!("Found {} playlists", library.playlists.len());
    library.derive_artist_album_playcounts();

    if config.update_navidrome {
        match std::fs::copy(
            &config.navidrome_import_database,
            &config.navidrome_export_database,
        ) {
            Err(_) => {
                log::error!("Failed to create a copy of the navidrome database for export");
                log::error!("Exiting without any further action.");
                std::process::exit(1)
            }
            Ok(_) => {
                log::info!("A copy of the navidrome database has made.");
            }
        };

        let writer =
            NavidromeWriter::from(std::path::Path::new(&config.navidrome_export_database))?;
        let user_id = writer.get_navidrome_user_id(&config);

        writer.update_tracks(&library, &user_id, &config);

        match writer.set_artist_album_counts(&library, &user_id) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error updating artist counts:\n{e:?}");
            }
        };
    }

    if config.apple_music_library_export_json {
        match library.json_export(&config.apple_music_library_json_export_path) {
            Ok(_) => {
                log::info!("Apple music library json export ok");
            }
            Err(e) => {
                log::error!("Error when exporting apple music library to JSON\n{e:?}")
            }
        }
    }

    if config.export_apple_music_playlists {
        export_playlists(&library, &config);
    }

    Ok(())
}

pub fn export_playlists(library: &Library, config: &Config) {
    if !std::fs::exists(&config.apple_music_playlist_export_directory).unwrap_or(true) {
        match std::fs::create_dir(&config.apple_music_playlist_export_directory) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Could not create directory for playlists.");
                log::error!("{e:?}");
                return;
            }
        }
    }

    for playlist in &library.playlists {
        if config
            .apple_music_ignored_playlists
            .iter()
            .any(|l| *l == playlist.name)
            || playlist.folder
        {
            continue;
        }
        log::trace!("Creating playlist: {}", playlist.name);
        match playlist.export_m3u(
            &config.apple_music_playlist_export_directory,
            &library.tracks,
        ) {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Error when creating playlist {}:", playlist.name);
                log::warn!("{e:?}");
            }
        };
    }
}
