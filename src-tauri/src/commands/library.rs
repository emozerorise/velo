use crate::storage::library::{
    LibraryData, LibraryError, LibraryMutation, LibraryResult, LibraryStore,
};
use std::sync::Mutex;
pub struct LibraryState(pub Mutex<LibraryStore>);
#[tauri::command]
pub fn library_load(state: tauri::State<'_, LibraryState>) -> LibraryResult<LibraryData> {
    state
        .0
        .lock()
        .map_err(|_| LibraryError::new("storage", "Library lock unavailable"))?
        .load()
}
#[tauri::command]
pub fn library_mutate(
    state: tauri::State<'_, LibraryState>,
    expected_revision: u64,
    mutation: LibraryMutation,
) -> LibraryResult<LibraryData> {
    state
        .0
        .lock()
        .map_err(|_| LibraryError::new("storage", "Library lock unavailable"))?
        .mutate(expected_revision, mutation)
}
#[tauri::command]
pub fn library_media_exists(path: String) -> bool {
    std::path::Path::new(&path).is_file()
}
