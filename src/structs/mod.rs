use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use playlist::Playlist;
use track::Track;

pub mod playlist;
pub mod track;

pub type TrackID = usize;
pub type TrackMap = std::collections::HashMap<TrackID, Track>;
type Artist = String;
type Album = String;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Library {
    pub tracks: HashMap<TrackID, Track>,
    pub playlists: Vec<Playlist>,
    pub counts: HashMap<Artist, ArtistCount>
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistCount {
    pub count: usize,
    pub albums: HashMap<Album, usize>,
}

impl Library {
    pub fn artist_album_playcounts(&mut self) {
        'track_loop: for track in self.tracks.values() {
            let artist = match &track.artist {
                Some(found) => found,
                None => continue 'track_loop,
            };
            let artist_entry = self.counts
                .entry(artist.to_owned())
                .or_default();
            artist_entry.count += track.play_count;
            if let Some(album) = &track.album_title {
                let ac = artist_entry
                    .albums
                    .entry(album.to_owned())
                    .or_insert(track.play_count);
                *artist_entry.albums.get_mut(album).unwrap() = std::cmp::min(track.play_count, *ac);
            }
        }
    }
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
