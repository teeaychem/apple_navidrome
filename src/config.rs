use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub apple_music_library: PathBuf,
    pub apple_music_library_export_json: bool,
    pub apple_music_library_json_export_path: PathBuf,

    pub export_apple_music_playlists: bool,
    pub apple_music_playlist_export_directory: PathBuf,
    pub apple_music_ignored_playlists: Vec<String>,

    pub update_navidrome: bool,
    pub navidrome_import_database: PathBuf,
    pub navidrome_export_database: PathBuf,
    pub navidrome_user: String,
    pub navidrome_user_id: Option<String>,

    pub record_failed_matches: bool,
    pub info_folder: PathBuf,
    pub no_match_file: PathBuf,
    pub multiple_matches_file: PathBuf,

    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            apple_music_library: PathBuf::from_str("Library.xml").unwrap(),
            apple_music_library_export_json: true,
            apple_music_library_json_export_path: PathBuf::from_str("Library.json").unwrap(),

            export_apple_music_playlists: true,
            apple_music_playlist_export_directory: PathBuf::from_str("playlists").unwrap(),
            apple_music_ignored_playlists: Vec::from([
                "Library".to_owned(),
                "Downloaded".to_owned(),
                "Music".to_owned(),
            ]),

            update_navidrome: true,
            navidrome_import_database: PathBuf::from_str("navidrome.db").unwrap(),
            navidrome_export_database: PathBuf::from_str("navidrome_updated.db").unwrap(),
            navidrome_user: "user".to_string(),
            navidrome_user_id: None,

            record_failed_matches: true,
            info_folder: PathBuf::from_str("info").unwrap(),
            no_match_file: PathBuf::from_str("no_matches.json").unwrap(),
            multiple_matches_file: PathBuf::from_str("multiple_matches.json").unwrap(),

            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_file() -> Config {
        let file_string = match std::fs::read_to_string("an_config.toml") {
            Ok(contents) => contents,
            Err(_) => {
                if !std::fs::exists("an_config.toml").unwrap_or(true) {
                    let mut file = match std::fs::File::create("an_config.toml") {
                        Ok(ok) => ok,
                        Err(e) => {
                            log::error!("Could create a default config file.");
                            log::error!("Error: {e:?}");
                            std::process::exit(1);
                        }
                    };
                    let default_config = Config::default();
                    let config_toml = toml::to_string_pretty(&default_config).unwrap();

                    match std::io::Write::write_all(&mut file, config_toml.as_bytes()) {
                        Ok(_) => {
                            log::info!("A default config file has been created.");
                            log::info!("Please check the contents of the file are okay and then run this program again.");
                            std::process::exit(0);
                        }
                        Err(e) => {
                            log::error!("Could write to the default config file after creating.");
                            log::error!("Error: {e:?}");
                            std::process::exit(1);
                        }
                    };
                } else {
                    log::warn!("There's some issue reading the config file, perhaps delete it and try again?");
                    std::process::exit(1);
                }
            }
        };
        let config: Config = match toml::from_str(&file_string) {
            Ok(toml) => toml,
            Err(e) => {
                log::error!("There was a problem reading the configuration file.");
                log::error!("Try fixing the file, or deleting it and running this program again.");
                log::error!("Error details: {e:?}");
                std::process::exit(1);
            }
        };
        config
    }

    pub fn info_path(&self, path: &PathBuf) -> PathBuf {
        if !self.info_folder.exists() {
            if let Err(e) = std::fs::create_dir(&self.info_folder) {
                log::error!(
                    "The folder to store additional information could not be created:\n{e:?}"
                );
            }
        }
        self.info_folder.join(path)
    }

    pub fn get_log_level(&self) -> log::LevelFilter {
        match log::LevelFilter::from_str(&self.log_level) {
            Err(e) => {
                log::error!(
                    "Could not set the config option \"log_level\" to \"{}\".",
                    self.log_level
                );
                log::error!("Error: {e:?}");
                log::error!(
                "Options are: \"Off\", \"Error\", \"Warn\", \"Info\", \"Debug\", and \"Trace\"."
            );
                log::error!("Using default value \"Info\"");
                log::LevelFilter::Info
            }
            Ok(level) => level,
        }
    }
}
