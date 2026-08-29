//! Core library implementation for Velo video player.

pub mod commands;
pub mod errors;
pub mod platform;
pub mod player;
pub mod storage;

use commands::settings::StorageState;
use parking_lot::Mutex;
use player::PlayerManager;
use std::sync::Arc;
use storage::{HistoryStore, SettingsStore};
use tauri::Manager;
#[cfg(not(target_os = "macos"))]
use tauri::WebviewWindow;
use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    info!("Starting Velo Player...");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle();

            // Initialize storage
            let settings_store = Arc::new(SettingsStore::new(handle)?);
            let history_store = Arc::new(HistoryStore::new(handle)?);
            let history_data = history_store.load();

            let storage_state = StorageState {
                settings: settings_store,
                history: history_store,
                cached_history: Mutex::new(history_data),
            };
            app.manage(storage_state);

            // Initialize Player Manager
            let player = Arc::new(PlayerManager::new()?);
            app.manage(player.clone());

            // Video surface. On macOS mpv renders through the render API
            // into a GL view inside this window; elsewhere it embeds itself
            // into the native window handle.
            #[cfg(target_os = "macos")]
            {
                let main_window = app
                    .get_webview_window("main")
                    .ok_or("main window is missing")?;
                let ns_window_ptr = main_window.ns_window()?;

                // Order matters: the GL context must exist and be current
                // before mpv is initialized and the render context bound.
                unsafe { platform::macos::MacosPlatform::create_video_surface(ns_window_ptr)? };
                player.initialize(handle.clone(), None)?;
                unsafe {
                    player::render::create(player.raw_handle())?;
                    player::render::set_update_callback(on_render_update);
                }

                let resize_target = main_window.clone();
                main_window.on_window_event(move |event| {
                    if matches!(
                        event,
                        tauri::WindowEvent::Resized(_)
                            | tauri::WindowEvent::ScaleFactorChanged { .. }
                    ) {
                        if let Ok(ptr) = resize_target.ns_window() {
                            unsafe { platform::macos::MacosPlatform::sync_video_surface(ptr) };
                        }
                    }
                });
            }

            #[cfg(not(target_os = "macos"))]
            {
                let wid =
                    app.get_webview_window("main").and_then(
                        |main_window| match native_video_surface(&main_window) {
                            Ok(wid) => wid,
                            Err(e) => {
                                tracing::warn!("Native window surface setup warning: {}", e);
                                None
                            }
                        },
                    );
                player.initialize(handle.clone(), wid)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::player_load_file,
            commands::player_play,
            commands::player_pause,
            commands::player_toggle_pause,
            commands::player_stop,
            commands::player_seek,
            commands::player_seek_absolute,
            commands::player_set_volume,
            commands::player_set_mute,
            commands::player_set_speed,
            commands::player_set_aspect_ratio,
            commands::player_select_audio_track,
            commands::player_select_subtitle_track,
            commands::player_add_subtitle_file,
            commands::player_set_subtitle_delay,
            commands::player_set_audio_delay,
            commands::playlist_scan_directory,
            commands::settings_get_all,
            commands::settings_save,
            commands::history_get,
            commands::history_record,
        ])
        .run(tauri::generate_context!())
        .expect("error while running velo application");
}

/// mpv signals a new frame from one of its own threads; hop to the main
/// thread and mark the video view dirty.
#[cfg(target_os = "macos")]
unsafe extern "C" fn on_render_update(_ctx: *mut std::os::raw::c_void) {
    platform::macos::MacosPlatform::request_redraw();
}

/// Resolve the native window handle mpv should render into. Only platforms
/// where mpv honours `wid` use this; macOS goes through the render API.
#[cfg(not(target_os = "macos"))]
fn native_video_surface(
    window: &WebviewWindow,
) -> std::result::Result<Option<i64>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        let hwnd_ptr = window.hwnd()?;
        unsafe {
            return Ok(Some(
                platform::windows::WindowsPlatform::setup_video_surface(
                    hwnd_ptr.0 as *mut std::ffi::c_void,
                )?,
            ));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        Ok(None)
    }
}
