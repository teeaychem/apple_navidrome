use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{num::ParseIntError, time::Duration};

use super::TrackID;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track {
    pub album_artist: String,
    pub album_rating: usize,
    pub album_title: String,
    pub artist: String,
    pub compiltion: bool,
    pub composer: String,
    pub date_added: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub disc_count: usize,
    pub disc_number: usize,
    pub duration: Duration,
    pub genre: String,
    pub grouping: Option<String>,
    pub location: String,
    pub movement_number: Option<usize>,
    pub persistent_id: String,
    pub play_count: usize,
    pub play_date: Option<DateTime<Utc>>,
    pub rating: usize,
    pub release_data: DateTime<Utc>,
    pub skip_count: usize,
    pub skip_date: Option<DateTime<Utc>>,
    pub title: String,
    pub total_tracks: usize,
    pub track_number: usize,
    pub work: Option<String>,
    pub year: usize,
    pub id: TrackID,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TrackErr {
    ParseDateTime(chrono::ParseError),
    ParseInt(ParseIntError),
    UnexpectedKV { key: String, value: String },
}
