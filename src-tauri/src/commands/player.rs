//! Tauri IPC commands for player operations.

use crate::errors::{Result, VeloError};
use crate::player::viewport::{ClientMetrics, VideoViewportInput, ViewportState};
use crate::player::PlayerManager;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

#[derive(Default)]
pub struct VideoViewportState(pub Arc<Mutex<HashMap<String, ViewportState>>>);

#[tauri::command]
pub async fn player_set_video_viewport(
    window: tauri::WebviewWindow,
    state: State<'_, VideoViewportState>,
    input: VideoViewportInput,
) -> Result<u64> {
    let states = state.0.clone();
    let ui_window = window.clone();
    let (sender, receiver) = tokio::sync::oneshot::channel();
    window
        .run_on_main_thread(move || {
            let result = (|| {
                let scale = ui_window
                    .scale_factor()
                    .map_err(|e| VeloError::Platform(e.to_string()))?;
                let size = ui_window
                    .inner_size()
                    .map_err(|e| VeloError::Platform(e.to_string()))?
                    .to_logical::<f64>(scale);
                let mut states = states.lock();
                let state = states.entry(ui_window.label().to_string()).or_default();
                let resolved = state.apply(
                    input,
                    ClientMetrics {
                        width: size.width,
                        height: size.height,
                        scale_factor: scale,
                    },
                )?;
                Ok(resolved.map_or_else(|| state.revision(), |viewport| viewport.revision))
            })();
            let _ = sender.send(result);
        })
        .map_err(|e| VeloError::Platform(e.to_string()))?;
    receiver
        .await
        .map_err(|_| VeloError::Platform("Video viewport window closed".into()))?
}

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
pub async fn player_set_volume(player: State<'_, Arc<PlayerManager>>, volume: f64) -> Result<()> {
    player.set_volume(volume)
}

#[tauri::command]
pub async fn player_set_mute(player: State<'_, Arc<PlayerManager>>, muted: bool) -> Result<()> {
    player.set_mute(muted)
}

#[tauri::command]
pub async fn player_set_speed(player: State<'_, Arc<PlayerManager>>, speed: f64) -> Result<()> {
    player.set_speed(speed)
}

#[tauri::command]
pub async fn player_set_aspect_ratio(
    player: State<'_, Arc<PlayerManager>>,
    ratio: String,
) -> Result<()> {
    player.set_aspect_ratio(&ratio)
}
