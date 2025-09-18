use std::cmp::Ordering;

use indexmap::IndexMap;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct Record {
    #[serde(rename = "Track name")]
    pub track_name: String,
    #[serde(rename = "Artist name")]
    pub artist_name: String,
    #[serde(rename = "Album")]
    pub album: String,
    #[serde(rename = "Playlist name")]
    pub playlist_name: String,
    #[serde(rename = "Type")]
    pub r#type: String,
    #[serde(rename = "ISRC")]
    pub isrc: String,
    #[serde(rename = "Tidal - id")]
    pub tidal_id: Option<String>,
    #[serde(rename = "Spotify - id")]
    pub spotify_id: Option<String>,
    #[serde(rename = "Qobuz - id")]
    pub qobuz_id: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Album {
    pub artist: String,
    pub title: String,
}

impl Ord for Album {
    fn cmp(&self, other: &Self) -> Ordering {
        self.artist
            .to_lowercase()
            .cmp(&other.artist.to_lowercase())
            .then(self.title.to_lowercase().cmp(&other.title.to_lowercase()))
    }
}

impl PartialOrd for Album {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Track {
    pub artist: String,
    pub album: String,
    pub title: String,
}

impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        self.artist
            .to_lowercase()
            .cmp(&other.artist.to_lowercase())
            .then(self.album.to_lowercase().cmp(&other.album.to_lowercase()))
            .then(self.title.to_lowercase().cmp(&other.title.to_lowercase()))
    }
}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Mappings {
    pub artists: IndexMap<String, String>,
    pub albums: IndexMap<String, Album>,
    pub tracks: IndexMap<String, Track>,
    pub playlists: IndexMap<String, IndexMap<String, Track>>,
}
