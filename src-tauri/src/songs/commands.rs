use tauri::State;

use super::models::Song;
use super::store::SongStore;

#[tauri::command]
pub fn list_songs(store: State<SongStore>) -> Vec<Song> {
    store.list()
}

#[tauri::command]
pub fn save_song(store: State<SongStore>, song: Song) -> Result<Song, String> {
    store.save_song(song)
}

#[tauri::command]
pub fn delete_song(store: State<SongStore>, id: u64) -> Result<(), String> {
    store.delete_song(id)
}
