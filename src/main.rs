mod models;

use std::{fs::File, path::Path};

use anyhow::Result;
use csv::Reader;
use indexmap::IndexMap;

use crate::models::{Album, Mappings, Record, Track};

fn build_mapping(path: impl AsRef<Path>) -> Result<Mappings> {
    let library = File::open(path)?;
    let mut reader = Reader::from_reader(library);

    let mut albums = IndexMap::new();
    let mut tracks = IndexMap::new();
    let mut playlists: IndexMap<String, IndexMap<String, Track>> = IndexMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        let isrc = record.isrc.trim_start_matches('0').to_uppercase();

        if record.r#type == "Album" {
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
        albums,
        tracks,
        playlists,
    })
}

fn compare_albums(
    spotify_albums: &IndexMap<String, Album>,
    tidal_albums: &IndexMap<String, Album>,
    qobuz_albums: &IndexMap<String, Album>,
) {
    for (reference_albums, other_albums, reference_name, other_name) in [
        (spotify_albums, tidal_albums, "Spotify", "TIDAL"),
        (spotify_albums, qobuz_albums, "Spotify", "Qobuz"),
    ] {
        let mut reference_albums_iter = reference_albums.iter();
        let mut current_reference_track = reference_albums_iter.next();

        for (index, (other_isrc, album)) in other_albums.iter().enumerate() {
            if !reference_albums.contains_key(other_isrc) {
                continue;
            }

            while let Some((reference_isrc, _)) = current_reference_track
                && !other_albums.contains_key(reference_isrc)
            {
                current_reference_track = reference_albums_iter.next();
            }

            let Some((reference_isrc, _)) = current_reference_track else {
                break;
            };

            if other_isrc != reference_isrc {
                println!(
                    "— [❌ {other_name} / ✔️ {reference_name}] #{}: {} - {}",
                    index + 1,
                    album.artist,
                    album.title
                );
                break;
            }

            current_reference_track = reference_albums_iter.next();
        }
    }

    let mut missing_albums = tidal_albums
        .iter()
        .filter(|(isrc, _)| !spotify_albums.contains_key(isrc.as_str()))
        .collect::<Vec<_>>();
    missing_albums.sort_by(|a, b| a.1.cmp(b.1));

    if !missing_albums.is_empty() {
        for (isrc, album) in missing_albums {
            println!(
                "— [➖ Spotify / ➕ TIDAL] {isrc} {} - {}",
                album.artist, album.title
            );
        }
    }

    let mut missing_albums = spotify_albums
        .iter()
        .filter(|(isrc, _)| !tidal_albums.contains_key(isrc.as_str()))
        .collect::<Vec<_>>();
    missing_albums.sort_by(|a, b| a.1.cmp(b.1));

    if !missing_albums.is_empty() {
        for (isrc, album) in missing_albums {
            println!(
                "— [➖ TIDAL / ➕ Spotfiy] {isrc} {} - {}",
                album.artist, album.title
            );
        }
    }

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

fn compare_tracks(
    spotify_tracks: &IndexMap<String, Track>,
    tidal_tracks: &IndexMap<String, Track>,
    qobuz_tracks: &IndexMap<String, Track>,
) {
    for (reference_tracks, other_tracks, reference_name, other_name) in [
        (spotify_tracks, tidal_tracks, "Spotify", "TIDAL"),
        (spotify_tracks, qobuz_tracks, "Spotify", "Qobuz"),
    ] {
        let mut reference_tracks_iter = reference_tracks.iter();
        let mut current_reference_track = reference_tracks_iter.next();

        for (index, (other_isrc, track)) in other_tracks.iter().enumerate() {
            if !reference_tracks.contains_key(other_isrc) {
                continue;
            }

            while let Some((reference_isrc, _)) = current_reference_track
                && !other_tracks.contains_key(reference_isrc)
            {
                current_reference_track = reference_tracks_iter.next();
            }

            let Some((reference_isrc, _)) = current_reference_track else {
                break;
            };

            if other_isrc != reference_isrc {
                println!(
                    "— [❌ {other_name} / ✔️ {reference_name}] #{}: {} - {} / {}",
                    index + 1,
                    track.artist,
                    track.album,
                    track.title
                );
                break;
            }

            current_reference_track = reference_tracks_iter.next();
        }
    }

    for (reference_tracks, other_tracks, reference_name, other_name) in [
        (tidal_tracks, spotify_tracks, "TIDAL", "Spotify"),
        (spotify_tracks, tidal_tracks, "Spotify", "TIDAL"),
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
    let mut spotify_mapping = build_mapping("My Spotify Library.csv").unwrap();
    let tidal_mapping = build_mapping("My TIDAL Library.csv").unwrap();
    let qobuz_mapping = build_mapping("My Qobuz Library.csv").unwrap();

    for (spotify_isrc, hires_isrc) in [
        ("859727497927", "859723464039"),
        ("886446990354", "886446990347"),
    ] {
        let index = spotify_mapping.albums.get_index_of(spotify_isrc).unwrap();
        spotify_mapping
            .albums
            .replace_index(index, hires_isrc.to_string())
            .unwrap();
    }

    println!();
    println!("Comparison of Favourite Albums");
    compare_albums(
        &spotify_mapping.albums,
        &tidal_mapping.albums,
        &qobuz_mapping.albums,
    );

    println!();
    println!("Comparison of Favourite Tracks");
    compare_tracks(
        &spotify_mapping.tracks,
        &tidal_mapping.tracks,
        &qobuz_mapping.tracks,
    );

    for (name, spotify_tracks) in &spotify_mapping.playlists {
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
        compare_tracks(spotify_tracks, tidal_tracks, qobuz_tracks);
    }
}
