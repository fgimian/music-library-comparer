mod models;

use std::{fs::File, path::Path};

use anyhow::Result;
use csv::Reader;
use indexmap::IndexMap;

use crate::models::{Album, Mappings, Record, Track};

fn build_mapping(path: impl AsRef<Path>) -> Result<Mappings> {
    let library = File::open(path)?;
    let mut reader = Reader::from_reader(library);

    let mut artists = IndexMap::new();
    let mut albums = IndexMap::new();
    let mut tracks = IndexMap::new();
    let mut playlists: IndexMap<String, IndexMap<String, Track>> = IndexMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        let isrc = record.isrc.trim_start_matches('0').to_uppercase();

        if record.r#type == "Artist" {
            artists.insert(
                record.track_name.to_lowercase().trim().to_string(),
                record.track_name.trim().to_string(),
            );
        } else if record.r#type == "Album" {
            albums.insert(
                isrc,
                Album {
                    artist: record.artist_name.to_string(),
                    title: record.album.to_string(),
                },
            );
        } else if record.r#type == "Favorite" {
            tracks.insert(
                isrc,
                Track {
                    artist: record.artist_name.to_string(),
                    album: record.album.to_string(),
                    title: record.track_name.to_string(),
                },
            );
        } else {
            let playlist = playlists.entry(record.playlist_name).or_default();
            playlist.insert(
                isrc,
                Track {
                    artist: record.artist_name.to_string(),
                    album: record.album.to_string(),
                    title: record.track_name.to_string(),
                },
            );
        }
    }

    Ok(Mappings {
        artists,
        albums,
        tracks,
        playlists,
    })
}

fn compare_artists(
    tidal_artists: &IndexMap<String, String>,
    qobuz_artists: &IndexMap<String, String>,
) {
    let mut missing_artists = tidal_artists
        .iter()
        .filter(|(tidal_artist_lowercase, _)| {
            !qobuz_artists.contains_key(tidal_artist_lowercase.as_str())
                && !qobuz_artists.iter().any(|(qobuz_artist_lowercase, _)| {
                    tidal_artist_lowercase.starts_with(qobuz_artist_lowercase)
                })
        })
        .collect::<Vec<_>>();
    missing_artists.sort_by(|a, b| a.1.cmp(b.1));

    if !missing_artists.is_empty() {
        for (_, artist) in missing_artists {
            println!("— [➖ Qobuz / ➕ TIDAL] - {artist}");
        }
    }

    let mut missing_artists = qobuz_artists
        .iter()
        .filter(|(qobuz_artist_lowercase, _)| {
            !tidal_artists.contains_key(qobuz_artist_lowercase.as_str())
                && !tidal_artists.iter().any(|(tidal_artist_lowercase, _)| {
                    tidal_artist_lowercase.starts_with(qobuz_artist_lowercase.as_str())
                })
        })
        .collect::<Vec<_>>();
    missing_artists.sort_by(|a, b| a.1.cmp(b.1));

    if !missing_artists.is_empty() {
        for (_, artist) in missing_artists {
            println!("— [➖ TIDAL / ➕ Qobuz] - {artist}");
        }
    }
}

fn compare_albums(tidal_albums: &IndexMap<String, Album>, qobuz_albums: &IndexMap<String, Album>) {
    let mut missing_albums = tidal_albums
        .iter()
        .filter(|(tidal_isrc, _)| {
            !qobuz_albums.contains_key(tidal_isrc.as_str())
                && !qobuz_albums
                    .iter()
                    .any(|(qobuz_isrc, _)| tidal_isrc.starts_with(qobuz_isrc))
        })
        .collect::<Vec<_>>();
    missing_albums.sort_by(|a, b| a.1.cmp(b.1));

    if !missing_albums.is_empty() {
        for (isrc, album) in missing_albums {
            println!(
                "— [➖ Qobuz / ➕ TIDAL] {isrc} {} - {}",
                album.artist, album.title
            );
        }
    }

    let mut missing_albums = qobuz_albums
        .iter()
        .filter(|(qobuz_isrc, _)| {
            !tidal_albums.contains_key(qobuz_isrc.as_str())
                && !tidal_albums
                    .iter()
                    .any(|(tidal_isrc, _)| tidal_isrc.starts_with(qobuz_isrc.as_str()))
        })
        .collect::<Vec<_>>();
    missing_albums.sort_by(|a, b| a.1.cmp(b.1));

    if !missing_albums.is_empty() {
        for (isrc, album) in missing_albums {
            println!(
                "— [➖ TIDAL / ➕ Qobuz] {isrc} {} - {}",
                album.artist, album.title
            );
        }
    }
}

fn compare_tracks(tidal_tracks: &IndexMap<String, Track>, qobuz_tracks: &IndexMap<String, Track>) {
    let mut reference_tracks_iter = tidal_tracks.iter();
    let mut current_reference_track = reference_tracks_iter.next();

    for (index, (other_isrc, track)) in qobuz_tracks.iter().enumerate() {
        if !tidal_tracks.contains_key(other_isrc) {
            continue;
        }

        while let Some((reference_isrc, _)) = current_reference_track
            && !qobuz_tracks.contains_key(reference_isrc)
        {
            current_reference_track = reference_tracks_iter.next();
        }

        let Some((reference_isrc, _)) = current_reference_track else {
            break;
        };

        if other_isrc != reference_isrc {
            println!(
                "— [❌ Qobuz / ✔️ TIDAL] #{}: {} - {} / {}",
                index + 1,
                track.artist,
                track.album,
                track.title
            );
            break;
        }

        current_reference_track = reference_tracks_iter.next();
    }

    for (reference_tracks, other_tracks, reference_name, other_name) in [
        (tidal_tracks, qobuz_tracks, "TIDAL", "Qobuz"),
        (qobuz_tracks, tidal_tracks, "Qobuz", "TIDAL"),
    ] {
        for (reference_isrc, track) in reference_tracks {
            if !other_tracks.contains_key(reference_isrc) {
                println!(
                    "— [➖ {other_name} / ➕ {reference_name}] {} {} - {} / {}",
                    reference_isrc, track.artist, track.album, track.title
                );
            }
        }
    }
}

fn main() {
    let tidal_mapping = build_mapping("My TIDAL Library.csv").unwrap();
    let qobuz_mapping = build_mapping("My Qobuz Library.csv").unwrap();

    println!("Comparison of Favourite Artists");
    compare_artists(&tidal_mapping.artists, &qobuz_mapping.artists);

    println!();
    println!("Comparison of Favourite Albums");
    compare_albums(&tidal_mapping.albums, &qobuz_mapping.albums);

    println!();
    println!("Comparison of Favourite Tracks");
    compare_tracks(&tidal_mapping.tracks, &qobuz_mapping.tracks);

    for name in tidal_mapping.playlists.keys() {
        let Some(tidal_tracks) = tidal_mapping.playlists.get(name) else {
            println!();
            println!("Comparison of Playlist: {name} - Missing on TIDAL, skipping!");
            continue;
        };

        let Some(qobuz_tracks) = qobuz_mapping.playlists.get(name) else {
            println!();
            println!("Comparison of Playlist: {name} - Missing on Qobuz, skipping!");
            continue;
        };

        println!();
        println!("Comparison of Playlist: {name}");
        compare_tracks(tidal_tracks, qobuz_tracks);
    }
}
