//! Tauri IPC commands for player operations.

use crate::errors::Result;
use crate::player::PlayerManager;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn player_load_file(
    player: State<'_, Arc<PlayerManager>>,
    path: String,
    start_time: Option<f64>,
) -> Result<()> {
    player.load_file(&path, start_time)
}

#[tauri::command]
pub async fn player_play(player: State<'_, Arc<PlayerManager>>) -> Result<()> {
    player.play()
}

#[tauri::command]
pub async fn player_pause(player: State<'_, Arc<PlayerManager>>) -> Result<()> {
    player.pause()
}

#[tauri::command]
pub async fn player_toggle_pause(player: State<'_, Arc<PlayerManager>>) -> Result<()> {
    player.toggle_pause()
}

#[tauri::command]
pub async fn player_stop(player: State<'_, Arc<PlayerManager>>) -> Result<()> {
    player.stop()
}

#[tauri::command]
pub async fn player_seek(
    player: State<'_, Arc<PlayerManager>>,
    seconds: f64,
    exact: bool,
) -> Result<()> {
    player.seek(seconds, exact)
}

#[tauri::command]
pub async fn player_seek_absolute(
    player: State<'_, Arc<PlayerManager>>,
    seconds: f64,
) -> Result<()> {
    player.seek_absolute(seconds)
}

#[tauri::command]
pub async fn player_set_volume(
    player: State<'_, Arc<PlayerManager>>,
    volume: f64,
) -> Result<()> {
    player.set_volume(volume)
}

#[tauri::command]
pub async fn player_set_mute(
    player: State<'_, Arc<PlayerManager>>,
    muted: bool,
) -> Result<()> {
    player.set_mute(muted)
}

#[tauri::command]
pub async fn player_set_speed(
    player: State<'_, Arc<PlayerManager>>,
    speed: f64,
) -> Result<()> {
    player.set_speed(speed)
}

#[tauri::command]
pub async fn player_set_aspect_ratio(
    player: State<'_, Arc<PlayerManager>>,
    ratio: String,
) -> Result<()> {
    player.set_aspect_ratio(&ratio)
}
