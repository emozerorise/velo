pub mod core;
pub mod events;
pub mod ffi;
pub mod manager;
// The render API path is macOS-only: it resolves GL symbols through dlopen,
// and every other platform embeds mpv with `wid` instead.
#[cfg(target_os = "macos")]
pub mod render;
pub mod types;

pub use manager::PlayerManager;
