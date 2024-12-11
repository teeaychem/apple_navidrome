use track::Track;

pub mod playlist;
pub mod track;

pub type TrackID = usize;
pub type TrackMap = std::collections::HashMap<TrackID, Track>;
