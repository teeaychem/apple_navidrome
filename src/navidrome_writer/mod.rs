use crate::{
    config::Config,
    structs::{track::Track, Library},
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, ToSql};

pub struct TrackMatcher<'t> {
    pub track: &'t Track,
    pub selections: Vec<&'t str>,
    pub binds: Vec<&'t str>,
    pub parameters: Vec<(&'t str, &'t dyn ToSql)>,
    pub item_id: Option<String>,
}

pub struct NavidromeWriter {
    pub db: Connection,
}

impl Drop for NavidromeWriter {
    fn drop(&mut self) {
        let mut tmp = Connection::open_in_memory().unwrap();
        std::mem::swap(&mut self.db, &mut tmp);
        match tmp.close() {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "An error occurred when trying to save the updated navidrome database.
Navidrome might not mind if the database has been corrupted, but be careful.
{e:?}"
                );
            }
        };
    }
}

impl<'t> TrackMatcher<'t> {
    pub fn from_track(track: &'t Track) -> Self {
        let mut identifier = TrackMatcher {
            track,
            selections: vec![],
            binds: vec![],
            parameters: vec![],
            item_id: None,
        };

        if let Some(artist) = &track.artist {
            identifier.selections.push("artist");
            identifier.parameters.push((":artist", artist));
            identifier.binds.push("artist = :artist");
        }

        if let Some(album) = &track.album_title {
            identifier.selections.push("album");
            identifier.parameters.push((":album", album));
            identifier.binds.push("album = :album");
        }

        // to ensure the formatted string lives suffiently long, if used
        // the like is used as (at least sometimes) without a track apple music uses the filename while navidrome uses a path
        // as the filename is included in the path, things work out
        let track_hack = match &track.title {
            Some(t) => Box::leak(Box::new(format!("%{t}"))),
            None => Box::leak(Box::new("%".to_string())),
        };
        if let Some(_use_hack) = &track.title {
            identifier.selections.push("title");
            identifier.parameters.push((":title", track_hack));
            identifier.binds.push("title LIKE :title");
        }

        if let Some(track_number) = &track.track_number {
            identifier.selections.push("track_number");
            identifier.parameters.push((":track_number", track_number));
            identifier.binds.push("track_number = :track_number");
        }

        if let Some(disc_number) = &track.disc_number {
            identifier.selections.push("disc_number");
            identifier.parameters.push((":disc_number", disc_number));
            identifier.binds.push("disc_number = :disc_number");
        }

        identifier
    }

    pub fn selects(&self) -> String {
        self.selections.join(", ")
    }

    pub fn binds(&self) -> String {
        self.binds.join(" AND ")
    }

    pub fn parameters(&self) -> &[(&'t str, &'t dyn ToSql)] {
        self.parameters.as_slice()
    }
}

impl NavidromeWriter {
    pub fn from(path: &std::path::Path) -> Result<Self, rusqlite::Error> {
        let connection = Connection::open(path)?;
        Ok(NavidromeWriter { db: connection })
    }

    pub fn item_ids(&self, matcher: &mut TrackMatcher) -> Result<Vec<String>, rusqlite::Error> {
        let mut item_ids: Vec<String> = vec![];

        let query_string = format!(
            "SELECT id, {} FROM media_file WHERE {}",
            matcher.selects(),
            matcher.binds()
        );

        let mut stmt = self.db.prepare(&query_string)?;
        let mut rows = stmt.query(matcher.parameters())?;
        while let Some(row) = rows.next()? {
            let id: Option<String> = row.get("id")?;
            if let Some(found) = id {
                item_ids.push(found.clone());
                matcher.item_id = Some(found);
            }
        }

        Ok(item_ids)
    }

    pub fn artist_id(&self, artist: &str) -> Result<Option<String>, rusqlite::Error> {
        let query_string = "SELECT id, name FROM artist WHERE name = :name";

        let mut stmt = self.db.prepare(query_string)?;
        let mut rows = stmt.query(&[(":name", artist)])?;
        while let Some(row) = rows.next()? {
            let id: Option<String> = row.get("id")?;
            if let Some(found) = id {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }

    pub fn album_id(
        &self,
        album: &str,
        artist_id: &str,
    ) -> Result<Option<String>, rusqlite::Error> {
        let query_string =
            "SELECT id, name, artist_id FROM album WHERE name = :name AND artist_id = :artist_id";

        let mut stmt = self.db.prepare(query_string)?;
        let mut rows = stmt.query(&[(":name", album), (":artist_id", artist_id)])?;
        while let Some(row) = rows.next()? {
            let id: Option<String> = row.get("id")?;
            if let Some(found) = id {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }

    const UPDATE_SCHEMA: &'static str = "
INSERT OR REPLACE INTO
annotation
(user_id, item_id, item_type, play_count, play_date, rating, starred, starred_at)
VALUES
(
:user_id,
:item_id,
:item_type,
:play_count,
:play_date,
:rating,
:starred,
:starred_at
)
";

    pub fn update_match(
        &self,
        matcher: &TrackMatcher,
        user_id: &str,
    ) -> Result<(), rusqlite::Error> {
        let params: [(&str, &dyn ToSql); 8] = [
            (":user_id", &user_id),
            (":item_id", &matcher.item_id),
            (":item_type", &"media_file"),
            (":play_count", &matcher.track.play_count),
            (":play_date", &matcher.track.play_date),
            (":rating", &matcher.track.rating),
            (
                ":starred",
                &(matcher.track.loved || matcher.track.favourited),
            ),
            (":starred_at", &None::<DateTime<Utc>>),
        ];

        let mut stmt = self.db.prepare(Self::UPDATE_SCHEMA)?;
        match stmt.execute(&params) {
            Err(e) => {
                let id = &matcher.item_id;
                log::error!("Error updating track {id:?}: {e:?}");
                Ok(())
            }
            Ok(_) => Ok(()),
        }
    }

    pub fn set_artist_album_counts(
        &self,
        library: &Library,
        user_id: &str,
    ) -> Result<(), rusqlite::Error> {
        'artist_loop: for (artist, counts) in &library.counts {
            match self.artist_id(artist.as_str()) {
                Ok(Some(artist_id)) => {
                    self.update_artist(&artist_id, counts.count, user_id)?;
                    for (album, count) in &counts.albums {
                        self.update_album(album, *count, &artist_id, user_id)?;
                    }
                }
                Ok(None) => {
                    log::trace!("Could not find an artist in the navidrome database: {artist}");
                    continue 'artist_loop;
                }
                Err(e) => {
                    log::error!("Failed to update artist: {:?}\n{e:?}", &artist);
                }
            }
        }
        Ok(())
    }

    pub fn update_artist(
        &self,
        artist_id: &str,
        count: usize,
        user_id: &str,
    ) -> Result<(), rusqlite::Error> {
        let params: [(&str, &dyn ToSql); 8] = [
            (":user_id", &user_id),
            (":item_id", &artist_id),
            (":item_type", &"artist"),
            (":play_count", &count),
            (":play_date", &None::<DateTime<Utc>>),
            (":rating", &None::<usize>),
            (":starred", &None::<bool>),
            (":starred_at", &None::<DateTime<Utc>>),
        ];

        let mut stmt = self.db.prepare(Self::UPDATE_SCHEMA)?;
        stmt.execute(&params)?;
        Ok(())
    }

    pub fn update_album(
        &self,
        album: &str,
        count: usize,
        artist_id: &str,
        user_id: &str,
    ) -> Result<(), rusqlite::Error> {
        match self.album_id(album, artist_id)? {
            Some(album_id) => {
                let params: [(&str, &dyn ToSql); 8] = [
                    (":user_id", &user_id),
                    (":item_id", &album_id),
                    (":item_type", &"album"),
                    (":play_count", &count),
                    (":play_date", &None::<DateTime<Utc>>),
                    (":rating", &None::<usize>),
                    (":starred", &None::<bool>),
                    (":starred_at", &None::<DateTime<Utc>>),
                ];
                let mut stmt = self.db.prepare(Self::UPDATE_SCHEMA)?;
                if let Err(e) = stmt.execute(&params) {
                    log::error!("Error updating album: {e:?}");
                }
            }
            None => {
                log::trace!("Could not find an album in the navidrome database: {album}");
            }
        }

        Ok(())
    }

    pub fn user_ids(&self, user: &str) -> Result<Vec<String>, rusqlite::Error> {
        let mut ids = vec![];

        let query_string = format!("SELECT id, user_name FROM user WHERE user_name = '{user}'");
        let mut stmt = self.db.prepare(&query_string)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id = row.get("id")?;
            ids.push(id);
        }

        Ok(ids)
    }

    pub fn update_tracks(&self, library: &Library, user_id: &str, config: &Config) {
        let mut failed_matches = vec![];
        let mut multiple_matches = vec![];
        for track in library.tracks.values() {
            let mut matcher = TrackMatcher::from_track(track);
            let ids = self.item_ids(&mut matcher).unwrap();
            match ids.len() {
                0 => failed_matches.push(track), // missing track
                1 => {
                    // unique track
                    match self.update_match(&matcher, user_id) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Failed to update track: {:?}\n{e:?}", &track.title);
                        }
                    };
                }
                _ => multiple_matches.push(track), // multiple tracks
            }
        }

        if !failed_matches.is_empty() {
            match write_failed_matches(failed_matches, config) {
            Ok(_) => {},
            Err(e) => log::warn!("Some tracks from Apple Music could not be matched to a track in the navidrome database.
An error occurred when attempting to write these to a file: {e:?}")
}
        }
        if !multiple_matches.is_empty() {
            match write_multiple_matches(multiple_matches, config) {
            Ok(_) => {},
            Err(e) => log::warn!("Some tracks from Apple Music were matched to multiple tracks in the navidrome database.
An error occurred when attempting to write these to a file: {e:?}")
}
        }
    }

    pub fn get_navidrome_user_id(&self, config: &Config) -> String {
        match &config.navidrome_user_id {
            Some(id) => {
                log::info!("user_id from config: {id}");
                id.clone()
            }
            None => {
                let user = &config.navidrome_user;
                let ids = match self.user_ids(&config.navidrome_user) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Failed to get possible user ids from the navidrome database");
                        log::error!("{e:?}");
                        std::process::exit(1);
                    }
                };
                match &ids[..] {
                    [] => {
                        log::error!("No user \"{user}\" found.");
                        std::process::exit(1);
                    }
                    [unique] => {
                        log::info!("User \"{user}\" found with id: {unique}");
                        unique.to_owned()
                    }
                    _ => {
                        log::error!("Multiple ids found for user \"{user}\".");
                        log::error!("Please add a database id to the config file");
                        log::error!("For example, a line which reads (though your ):");
                        log::error!("navidrome_user_id = \"d868f4b6-1d16-4c05-ae0c-4aca4ef42788\"");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

fn write_failed_matches(
    failed_matches: Vec<&Track>,
    config: &Config,
) -> Result<(), std::io::Error> {
    let mut fail_match_file = std::fs::File::create(config.info_path(&config.no_match_file))?;
    let _ = std::io::Write::write_all(
        &mut fail_match_file,
        serde_json::to_string_pretty(&failed_matches)?.as_bytes(),
    );
    log::warn!(
        "Some tracks from Apple Music could not be matched to a track in the navidrome database.
A file containing these tracks has been made."
    );
    Ok(())
}

fn write_multiple_matches(
    multiple_matches: Vec<&Track>,
    config: &Config,
) -> Result<(), std::io::Error> {
    let mut mismatch_file = std::fs::File::create(config.info_path(&config.multiple_matches_file))?;

    let _ = std::io::Write::write_all(
        &mut mismatch_file,
        serde_json::to_string_pretty(&multiple_matches)?.as_bytes(),
    );
    log::warn!(
        "Some tracks from Apple Music were matched to multiple tracks in the navidrome database.
A file containing these tracks has been made."
    );
    Ok(())
}
