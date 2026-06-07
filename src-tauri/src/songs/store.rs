use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::models::{Song, SongSection};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SongLibrary {
    songs: Vec<Song>,
}

pub struct SongStore {
    songs: Mutex<Vec<Song>>,
    path: Mutex<Option<PathBuf>>,
}

impl SongStore {
    pub fn new() -> Self {
        Self {
            songs: Mutex::new(default_songs()),
            path: Mutex::new(None),
        }
    }

    pub fn init(&self, app_dir: &Path) {
        let path = app_dir.join("songs.json");
        *self.path.lock().unwrap() = Some(path.clone());
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(lib) = serde_json::from_str::<SongLibrary>(&data) {
                if !lib.songs.is_empty() {
                    *self.songs.lock().unwrap() = lib.songs;
                    return;
                }
            }
        }
        let _ = self.save();
    }

    pub fn list(&self) -> Vec<Song> {
        self.songs.lock().unwrap().clone()
    }

    pub fn save_song(&self, song: Song) -> Result<Song, String> {
        let mut songs = self.songs.lock().unwrap();
        if let Some(pos) = songs.iter().position(|s| s.id == song.id) {
            songs[pos] = song.clone();
        } else {
            songs.insert(0, song.clone());
        }
        drop(songs);
        self.save()?;
        Ok(song)
    }

    pub fn delete_song(&self, id: u64) -> Result<(), String> {
        self.songs.lock().unwrap().retain(|s| s.id != id);
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let path = self.path.lock().unwrap().clone().ok_or("Song store not initialized")?;
        let songs = self.songs.lock().unwrap().clone();
        let json = serde_json::to_string_pretty(&SongLibrary { songs }).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }
}

fn default_songs() -> Vec<Song> {
    vec![
        Song {
            id: 1,
            title: "Amazing Grace".into(),
            artist: "John Newton".into(),
            ccli: Some("22025".into()),
            copyright: Some("Public Domain".into()),
            key: "G".into(),
            tempo: "72 BPM".into(),
            arrangement: vec!["v1".into(), "c1".into(), "v2".into(), "c1".into()],
            sections: vec![
                SongSection {
                    id: "v1".into(),
                    label: "Verse 1".into(),
                    lyrics: "Amazing grace, how sweet the sound\nThat saved a wretch like me".into(),
                },
                SongSection {
                    id: "c1".into(),
                    label: "Chorus".into(),
                    lyrics: "My chains are gone, I have been set free".into(),
                },
                SongSection {
                    id: "v2".into(),
                    label: "Verse 2".into(),
                    lyrics: "Through many dangers, toils, and snares\nI have already come".into(),
                },
            ],
        },
        Song {
            id: 2,
            title: "Way Maker".into(),
            artist: "Sinach".into(),
            ccli: Some("7115744".into()),
            copyright: Some("2016 Integrity Music Europe".into()),
            key: "Bb".into(),
            tempo: "68 BPM".into(),
            arrangement: vec!["v1".into(), "c1".into(), "b1".into(), "c1".into()],
            sections: vec![
                SongSection {
                    id: "v1".into(),
                    label: "Verse 1".into(),
                    lyrics: "You are here, moving in our midst\nI worship You, I worship You".into(),
                },
                SongSection {
                    id: "c1".into(),
                    label: "Chorus".into(),
                    lyrics: "Way maker, miracle worker\nPromise keeper, light in the darkness".into(),
                },
                SongSection {
                    id: "b1".into(),
                    label: "Bridge".into(),
                    lyrics: "Even when I do not see it, You are working".into(),
                },
            ],
        },
    ]
}
