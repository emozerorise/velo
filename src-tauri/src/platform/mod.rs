pub mod macos;
pub mod windows;

#[cfg(target_os = "macos")]
pub use macos::MacosPlatform as Platform;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as Platform;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub struct UnsupportedPlatform;
