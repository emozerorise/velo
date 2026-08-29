//! Tauri IPC commands for audio & subtitle tracks.

use crate::errors::Result;
use crate::player::PlayerManager;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn player_select_audio_track(
    player: State<'_, Arc<PlayerManager>>,
    id: i64,
) -> Result<()> {
    player.select_audio_track(id)
}

#[tauri::command]
pub async fn player_select_subtitle_track(
    player: State<'_, Arc<PlayerManager>>,
    id: i64,
) -> Result<()> {
    player.select_subtitle_track(id)
}

#[tauri::command]
pub async fn player_add_subtitle_file(
    player: State<'_, Arc<PlayerManager>>,
    path: String,
) -> Result<()> {
    player.add_subtitle_file(&path)
}

#[tauri::command]
pub async fn player_set_subtitle_delay(
    player: State<'_, Arc<PlayerManager>>,
    seconds: f64,
) -> Result<()> {
    player.set_subtitle_delay(seconds)
}

#[tauri::command]
pub async fn player_set_audio_delay(
    player: State<'_, Arc<PlayerManager>>,
    seconds: f64,
) -> Result<()> {
    player.set_audio_delay(seconds)
}
