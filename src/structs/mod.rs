use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use playlist::Playlist;
use track::Track;

pub mod playlist;
pub mod track;

pub type TrackID = usize;
pub type TrackMap = std::collections::HashMap<TrackID, Track>;


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct  Library {
    pub tracks: HashMap<TrackID, Track>,
    pub playlists: Vec<Playlist>
}
