use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{num::ParseIntError, time::Duration};

use super::TrackID;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track {
    pub album_artist: Option<String>,
    pub album_rating: Option<usize>,
    pub album_title: Option<String>,
    pub artist: Option<String>,
    pub bpm: Option<usize>,
    pub comments: Option<String>,
    pub compiltion: bool,
    pub composer: Option<String>,
    pub date_added: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub disc_count: Option<usize>,
    pub disc_number: Option<usize>,
    pub duration: Duration,
    pub genre: Option<String>,
    pub grouping: Option<String>,
    pub id: TrackID,
    pub location: String,
    pub movement_number: Option<usize>,
    pub movement_title: Option<String>,
    pub persistent_id: String,
    pub play_count: usize,
    pub play_date: Option<DateTime<Utc>>,
    pub rating: usize,
    pub release_data: Option<DateTime<Utc>>,
    pub size: usize,
    pub skip_count: usize,
    pub skip_date: Option<DateTime<Utc>>,
    pub title: Option<String>,
    pub total_tracks: Option<usize>,
    pub track_number: Option<usize>,
    pub work: Option<String>,
    pub year: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TrackErr {
    ParseDateTime(chrono::ParseError),
    ParseInt(ParseIntError),
    UnexpectedKV { key: String, value: String },
}
