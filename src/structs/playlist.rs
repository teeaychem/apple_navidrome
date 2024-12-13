use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::xml_reader::{self};

use super::*;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub description: String,
    pub persistent_id: String,
    pub parent_persistent_id: String,
    pub folder: bool,
    pub track_ids: Vec<TrackID>,
}

impl Playlist {
    pub fn export_m3u(
        &self,
        path: &Path,
        tracks: &TrackMap,
    ) -> Result<(), xml_reader::err::LibraryXmlReader> {
        let playlist_filename = format!("{}.m3u8", self.name);
        let playlist_path = path.join(Path::new(&playlist_filename));
        match File::create(playlist_path.clone()) {
            Ok(mut file) => {
                writeln!(file, "#EXTM3U")?;
                writeln!(file, "#PLAYLIST:{}", self.name)?;
                for id in &self.track_ids {
                    let track = match tracks.get(id) {
                        Some(t) => t,
                        None => {
                            return Err(xml_reader::err::LibraryXmlReader::MissingTrack {
                                playlist: self.name.to_owned(),
                                track_id: id.clone(),
                            })
                        }
                    };
                    let title = match &track.title {
                        Some(found) => found,
                        None => "[No title]",
                    };
                    let artist = match &track.artist {
                        Some(found) => found,
                        None => "[No artist]",
                    };
                    writeln!(
                        file,
                        "#EXTINF:{},{artist} - {title}",
                        track.duration.as_secs(),
                    )?;
                    let abs_pth = &track.location;
                    writeln!(file, "{}", abs_pth)?;
                }
            }
            Err(_) => {
                println!("Failed to create a file for playlist {}", self.name);
            }
        }
        Ok(())
    }
}
