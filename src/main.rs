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
    let mut qobuz_mapping = build_mapping("My Qobuz Library.csv").unwrap();

    for (spotify_isrc, hires_isrc) in [
        // Alexis Ffrench - Evolution
        ("886446990354", "886446990347"),
        // Alexis Ffrench - Truth
        ("886449668656", "886449839520"),
        // Carly Pearce - Carly Pearce
        ("843930050222", "843930050239"),
        // Chris Tomlin - Chris Tomlin & Friends
        ("602507408510", "602508788475"),
        // Daniil Trifonov - Chopin Evocations
        ("28947974741", "28947974765"),
        // Danny Gokey - Sound Of Heaven
        ("602465089790", "602465089806"),
        // Freya Ridings - Freya Ridings
        ("602577537288", "602577537318"),
        // George Frideric Handel / Emma Kirkby - Handel: Messiah (Remastered 2014)
        ("28947881650", "28947881674"),
        // Housefires - How To Start A Housefire
        ("602448696533", "602455166241"),
        // Jeremy Camp - The Story's Not Over
        ("602567427766", "602508465482"),
        // Jeremy Camp - When You Speak
        ("602508353871", "602507446925"),
        // Jeremy Rosado - The Waiting Room
        ("602478346743", "602478346750"),
        // Joe Hisaishi - NOSTALGIA ～PIANO STORIES III～
        ("602508731693", "602508731716"),
        // Jonathan Traylor - Closer Than You Think
        ("602445996179", "602445996186"),
        // Jordan Davis - Bluebird Days
        ("602455058966", "602455058935"),
        // Jordan Davis - Buy Dirt
        ("602438070404", "602438070411"),
        // Jordan Davis - Jordan Davis
        ("602508988004", "602508988011"),
        // Kacey Musgraves - star-crossed
        ("602438699216", "602438699223"),
        // Kari Jobe - The Blessing (Live)
        ("602557919509", "602507233655"),
        // Kari Jobe - The Blessing
        ("602577229503", "602508830488"),
        // Kim Walker-Smith - Wild Heart (Live)
        ("602557922066", "602507149062"),
        // Lady A - Ocean
        ("843930047079", "843930047574"),
        // Lauren Alaina - Getting Good
        ("602508742620", "602508742637"),
        // Lights - Midnight Machines
        ("93624921431", "93624921424"),
        // Ludwig van Beethoven / Wiener Philharmoniker - Beethoven: Symphonies Nos. 5 & 7
        ("28948637751", "28948638796"),
        // Maddie & Tae - The Way It Feels
        ("602508780288", "602508780295"),
        // Maddie & Tae - Through The Madness Vol. 1
        ("602445320400", "602445320417"),
        // Mosaic MSC - This Is How I Thank The Lord
        ("602438574841", "602438574865"),
        // Ola Gjeilo - Ola Gjeilo
        ("28947886914", "28947886938"),
        // Pat Barrett - Shelter
        ("602507447151", "602507447175"),
        // Riley Clemmons - Godsend (Deluxe)
        ("602438725328", "602438725342"),
        // Runaway June - Blue Roses
        ("4050538508703", "4050538508727"),
        // Sean Curran - 1,000 Names
        ("602438784820", "602438784844"),
        // Tasha Layton - How Far
        ("810539026620", "810539025272"),
        // Tauren Wells - Let The Church Sing
        ("602475821199", "602475821205"),
        // TAYA - TAYA
        ("602455038814", "602455046352"),
        // The Belonging Co - Now (Live)
        ("687398362660", "687398362684"),
        // The Belonging Co - See The Light (Live)
        ("687398362257", "687398362271"),
        // The Belonging Co - TEN
        ("850052903176", "850052903183"),
        // VOUS Worship - Dying To Be Different (Live)
        ("840468806208", "840468806192"),
        // VOUS Worship - Future Glory (Live)
        ("810116758876", "810116758869"),
        // VOUS Worship - I Need Revival (Live)
        ("7316470064712", "810116755813"),
        // Within Temptation - Resist (Extended Deluxe)
        ("602577347689", "602577347665"),
        // Yiruma - The Rewritten Memories
        ("602435607467", "602435607474"),
    ] {
        let index = spotify_mapping.albums.get_index_of(spotify_isrc).unwrap();
        spotify_mapping
            .albums
            .replace_index(index, hires_isrc.to_string())
            .unwrap();
    }

    for (qobuz_isrc, tidal_isrc) in [
        // Alma Deutscher - From My Book of Melodies
        ("886447893517", "886447893500"),
        // Arthur Rubinstein - Chopin: Nocturnes
        ("886443706675", "884977564013"),
        // HAUSER - Classic
        ("886447884034", "886447884010"),
        // Housefires - How To Start A Housefire
        ("602448696595", "602455166241"),
    ] {
        let index = qobuz_mapping.albums.get_index_of(qobuz_isrc).unwrap();
        qobuz_mapping
            .albums
            .replace_index(index, tidal_isrc.to_string())
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
