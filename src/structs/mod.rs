use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use playlist::Playlist;
use track::Track;

pub mod playlist;
pub mod track;

pub type TrackID = usize;
pub type TrackMap = std::collections::HashMap<TrackID, Track>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Library {
    pub tracks: HashMap<TrackID, Track>,
    pub playlists: Vec<Playlist>,
}

impl Library {
    pub fn json_export(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let library_json = serde_json::to_string(self).unwrap();
        let mut file = std::fs::File::create(path)?;
        std::io::Write::write_all(&mut file, library_json.as_bytes())?;
        Ok(())
    }

    pub fn from_json(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let library: Library = serde_json::from_reader(reader)?;
        Ok(library)
    }
}
