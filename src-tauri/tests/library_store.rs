use serde_json::json;
use velo_lib::storage::library::LibraryStore;
use velo_lib::storage::library::{apply_mutation, LibraryData, LibraryMutation};

fn test_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "velo-library-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
#[test]
fn library_persists_and_rejects_stale_revision() {
    let dir = test_dir();
    let mut store = LibraryStore::open(dir.clone());
    assert_eq!(store.load().unwrap().revision, 0);
    let saved = store
        .mutate(
            0,
            serde_json::from_value(json!({"type":"create_project","id":"p","name":"ประชุม"}))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(LibraryStore::open(dir.clone()).load().unwrap(), saved);
    let err = store
        .mutate(
            0,
            serde_json::from_value(json!({"type":"delete_project","id":"p"})).unwrap(),
        )
        .unwrap_err();
    assert_eq!(err.kind, "conflict");
    assert_eq!(store.load().unwrap(), saved);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn library_bad_file_is_not_overwritten() {
    for bytes in ["{broken".to_string(),serde_json::to_string(&json!({"version":99,"revision":0,"projects":[],"recordings":[],"preferences":{"collapsed_project_ids":[],"sort":"added"}})).unwrap()] {
        let dir=test_dir();std::fs::write(dir.join("library.json"),&bytes).unwrap();
        let mut store=LibraryStore::open(dir.clone()); assert!(store.load().is_err());
        assert!(store.mutate(0,serde_json::from_value(json!({"type":"create_project","id":"p","name":"x"})).unwrap()).is_err());
        assert_eq!(std::fs::read_to_string(dir.join("library.json")).unwrap(),bytes);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

fn apply(data: &LibraryData, value: serde_json::Value) -> Result<LibraryData, String> {
    let mutation: LibraryMutation = serde_json::from_value(value).unwrap();
    apply_mutation(data, mutation).map_err(|e| e.kind)
}
fn seeded() -> LibraryData {
    apply(&LibraryData::default(), json!({"type":"add_recordings","entries":[{"id":"a","path":fixture_path("recordings")},{"id":"b","path":fixture_path("other")}],"project_id":null})).unwrap()
}
fn fixture_path(folder: &str) -> String {
    // A drive-less Unix root is not an absolute path on Windows.
    std::env::temp_dir()
        .join(folder)
        .join("a.mp4")
        .to_string_lossy()
        .into_owned()
}
#[test]
fn library_names_and_identity() {
    let original = seeded();
    assert_eq!(original.recordings.len(), 2);
    let renamed = apply(
        &original,
        json!({"type":"rename_recording","id":"a","name":"  ประชุมทีม  "}),
    )
    .unwrap();
    assert_eq!(
        renamed.recordings[0].display_name.as_deref(),
        Some("ประชุมทีม")
    );
    assert_eq!(renamed.recordings[0].path, fixture_path("recordings"));
    assert_eq!(original.recordings[0].display_name, None);
    let again=apply(&renamed,json!({"type":"add_recordings","entries":[{"id":"new","path":fixture_path("recordings")}],"project_id":null})).unwrap();
    assert_eq!(again.recordings, renamed.recordings);
    for (name, valid) in [
        ("🙂".repeat(120), true),
        ("ก".repeat(121), false),
        ("  ".into(), false),
    ] {
        assert_eq!(
            apply(
                &renamed,
                json!({"type":"rename_recording","id":"a","name":name})
            )
            .is_ok(),
            valid
        );
    }
    let reset = apply(&renamed, json!({"type":"reset_name","id":"a"})).unwrap();
    assert_eq!(reset.recordings[0].display_name, None);
}
#[test]
fn library_project_rules() {
    let mut data = apply(
        &seeded(),
        json!({"type":"create_project","id":"p","name":"Team"}),
    )
    .unwrap();
    assert!(apply(
        &data,
        json!({"type":"create_project","id":"q","name":" team "})
    )
    .is_err());
    assert!(apply(
        &data,
        json!({"type":"move_recording","id":"a","project_id":"missing"})
    )
    .is_err());
    assert!(apply(
        &data,
        json!({"type":"rename_recording","id":"missing","name":"x"})
    )
    .is_err());
    data = apply(
        &data,
        json!({"type":"move_recording","id":"a","project_id":"p"}),
    )
    .unwrap();
    data = apply(&data, json!({"type":"set_pin","id":"a","pinned":true})).unwrap();
    data=apply(&data,json!({"type":"set_preferences","preferences":{"collapsed_project_ids":["p"],"sort":"name"}})).unwrap();
    data = apply(&data, json!({"type":"delete_project","id":"p"})).unwrap();
    assert_eq!(data.recordings.len(), 2);
    assert_eq!(data.recordings[0].project_id, None);
    assert!(data.recordings[0].pinned);
    assert!(data.preferences.collapsed_project_ids.is_empty());
    assert!(apply(&data,json!({"type":"add_recordings","entries":[{"id":"c","path":"relative.mp4"}],"project_id":null})).is_err());
    assert!(apply(&data, json!({"type":"set_duration","id":"a","duration":-1})).is_err());
}

#[test]
fn dropped_paths_expand_folders_and_skip_non_media() {
    let dir = test_dir();
    std::fs::write(dir.join("a.mp4"), []).unwrap();
    std::fs::write(dir.join("notes.txt"), []).unwrap();
    let result = velo_lib::commands::playlist_resolve_paths(vec![
        dir.to_string_lossy().into_owned(),
        dir.join("a.mp4").to_string_lossy().into_owned(),
    ])
    .unwrap();
    assert_eq!(
        result.files,
        vec![dir.join("a.mp4").to_string_lossy().into_owned()]
    );
    std::fs::remove_dir_all(dir).unwrap();
}
