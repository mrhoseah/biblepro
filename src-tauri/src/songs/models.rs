use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSection {
    pub id: String,
    pub label: String,
    pub lyrics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: u64,
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub ccli: Option<String>,
    #[serde(default)]
    pub copyright: Option<String>,
    pub key: String,
    pub tempo: String,
    pub arrangement: Vec<String>,
    pub sections: Vec<SongSection>,
}
