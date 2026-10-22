# Project Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Execution method: **Native**, selected by the user. If that skill is unavailable, report it and follow these checkboxes in this session with TDD and verification-before-completion. No parallel implementation agents.

**Goal:** Persist meaningful recording display names and project groups without changing media files, existing caches or playback behavior.

**Architecture:** Rust owns a versioned library store and serialized, revision-checked mutations. Pinia mirrors confirmed snapshots; browsing remains separate from the existing transient playlist. All ingestion paths share a library-aware coordinator, and all display surfaces share a name resolver.

**Tech Stack:** Existing Tauri v2, Rust/Serde, Vue 3/Pinia, TypeScript, Vitest and current UI/icon/font dependencies. No new database or cloud service.

**Spec:** `docs/superpowers/specs/2026-10-22-project-library-design.md` (approved by user on 2026-10-22).

## Global Constraints

- “No push or release.” Preserve all existing uncommitted work; do not stage entire files containing unrelated changes. No commit is required for this local implementation pass.
- “Never change the media path or extension.” No rename/move/copy/delete of original media.
- “Transcript/summary caches, chat context and history/resume identity continue to use the original path.” No summary API calls.
- “A recording belongs to zero or one project; nested projects are out of scope.” Pinned rows are shortcuts, not duplicate records.
- “reject empty names and names over 120 Unicode characters.” Count Unicode scalar values consistently with Rust chars and JS Array.from, after trim; reset is a distinct operation.
- “Search and sorting affect browsing only, never the running playback sequence.” Restoring the library never autoplays.
- “Do not close the user's running app without asking.” Restart verification uses only a separately launched isolated test instance.
- Existing themes, Noto Sans Thai and icons remain; all UI copy goes in en.ts and typed th.ts.

## Review Focus

1. A file-open event arrives while disk initialization is pending: wait for initialization, never replace saved records with an empty snapshot (task3).
2. Thai/emoji names and duplicate basenames: enforce the same length rule on both sides, distinguish by original path, never use display names as identity (tasks1/4).
3. A stale edit races another queued edit: reject stale revision, reload, preserve the user's draft and require explicit retry (tasks2/3).
4. Removing the currently playing recording or searching a different project: playback and queue identity stay unchanged (tasks3/5).
5. Unsupported/corrupt data or an external drive disappearing: retain bytes/records, show a recoverable error, and keep direct playback usable (tasks2/5).

## File ownership map

- New `src-tauri/src/storage/library/{mod.rs,types.rs,mutation.rs}`: schema, validation and mutation logic, disk persistence respectively organized by responsibility (mod.rs owns disk I/O).
- New `src-tauri/src/commands/library.rs`: Tauri bridge and managed mutex; register in existing commands/mod.rs and lib.rs. Export store through storage/mod.rs.
- New `src/types/library.ts`, `src/services/libraryService.ts`, `src/stores/libraryStore.ts`: typed wire model, invocation boundary, confirmed reactive snapshots.
- New `src/services/mediaIngestion.ts`: library registration + existing queue/play actions; no storage writes in player progress watchers.
- New `src/utils/libraryView.ts`: display names, sorting, grouping and search; pure and testable.
- New `src/components/library/{LibraryBrowser,LibraryGroup,LibraryRow,LibraryItemMenu,LibraryNameEditor}.vue`: focused browser, group, row, actions and editor/confirmation surfaces.
- Existing ReadingSidebar.vue hosts LibraryBrowser. Existing PlaylistItem.vue, PlaylistDrawer.vue, TopBar.vue and the active header/player dock bindings consume shared name resolution. Locate the latter bindings with `rg 'fileName|file_name|mediaPath' src/components src/App.vue` before editing; do not restructure summary components.
- New tests: `src-tauri/tests/library_store.rs`, `tests/unit/{libraryStore,libraryView,libraryBrowser,libraryPlayback}.test.ts`; extend existing openMedia.test.ts and keyboardShortcuts.test.ts.

## Shared contracts

JSON uses snake_case for Rust/TypeScript fields. `LibraryData` has `version:1`, `revision:number`, `projects:LibraryProject[]`, `recordings:LibraryRecording[]`, `preferences:LibraryPreferences`.

- Project: `id:string`, `name:string`, `created_at:number` (Unix milliseconds).
- Recording: `id:string`, `path:string`, `display_name:string|null`, `project_id:string|null`, `pinned:boolean`, `added_at:number`, `duration:number|null` (finite nonnegative seconds).
- Preferences: `collapsed_project_ids:string[]`, `sort:'added'|'played'|'name'`. Use reserved group key `uncategorized` only in collapse preferences, never as a project ID. Defaults: no collapsed groups, sort added.
- `LibraryMutation` is a Serde internally tagged union with `type`: `add_recordings {entries,project_id}` (entries are `{id:string,path:string}[]`), `create_project {id,name}`, `rename_project {id,name}`, `delete_project {id}`, `rename_recording {id,name}`, `reset_name {id}`, `move_recording {id,project_id}`, `set_pin {id,pinned}`, `remove_recording {id}`, `set_preferences {preferences}`, `set_duration {id,duration}`. IDs originate from crypto.randomUUID in the frontend. Rust treats them as opaque nonempty strings of at most128 ASCII characters and validates uniqueness; an existing path retains its stored ID regardless of the incoming proposed ID. No new UUID dependency.
- IPC: `library_load() -> Result<LibraryData, LibraryError>` and `library_mutate(expected_revision:u64, mutation:LibraryMutation) -> Result<LibraryData, LibraryError>`. Serializable error `{kind,message}`; kinds `storage`, `invalid`, `conflict`, `unsupported_version`.
- Rust: `LibraryStore::open(dir:PathBuf) -> Self`, `load(&mut self) -> Result<LibraryData, LibraryError>`, `mutate(&mut self, expected_revision:u64, mutation:LibraryMutation) -> Result<LibraryData, LibraryError>`. Opening stores only the path; a load failure does not prevent app startup. Mutation requires a successful load.
- Frontend: service `load():Promise<LibraryData>`, `mutate(revision:number, mutation:LibraryMutation):Promise<LibraryData>`; store `initialize():Promise<void>`, `retryLoad():Promise<void>`, `apply(mutation:LibraryMutation):Promise<boolean>`, `displayName(path:string):string`. Expose snapshot, loading, error and writesEnabled. A failed load settles initialization in error state rather than leaving ingestion awaiting forever.

### Task 1: Validated metadata and deterministic mutations

**Files:** Create storage/library/types.rs and mutation.rs, wire mod.rs + storage/mod.rs, create tests/library_store.rs.

**Interfaces:** Produce the shared schema, `LibraryData::default()` and pure `apply_mutation(data:&LibraryData, mutation:LibraryMutation) -> Result<LibraryData, LibraryError>`; disk writes come in task2. Generate timestamps/IDs only for genuinely new records.

- [ ] Write failing `library_names_and_identity` tests. Assertions: 120 scalar values accepted, 121 rejected; whitespace-only rejected; reset yields null; Thai/emoji remain intact; two different absolute paths with identical basename remain two records; same exact path twice preserves ID/name/project/pin; source path stays byte-for-byte identical.
- [ ] Write failing `library_project_rules` tests: case-insensitive trimmed duplicate project name rejected, duplicate recording labels allowed, dangling project/recording IDs rejected; deleting a project makes its recordings project_id=null and removes its collapse key; pinning does not increase recording count. Empty relative paths and nonfinite/negative duration rejected.
- [ ] Run `cd src-tauri && cargo test --test library_store`; expect new test failures before implementation, not unrelated build failures.
- [ ] Implement schema and mutation validation. Empty library defaults version1/revision0; immutable candidate construction leaves the input unchanged on every failure. Project name uniqueness uses trimmed Unicode lowercase on both sides. Name sorting may be locale-aware in UI; IDs/paths are never lowercased.
- [ ] Rerun the focused tests; expect all task1 cases passing. Review only task1 diff before proceeding.

### Task 2: Safe disk persistence and IPC

**Files:** storage/library/mod.rs, commands/library.rs, commands/mod.rs, lib.rs, tests/library_store.rs.

**Interfaces:** Produce LibraryStore and the two IPC commands from Shared contracts; manage one mutex-protected LibraryStore per app instance.

- [ ] Write failing `library_persists_across_reopen`, `library_failed_save_preserves_snapshot`, `library_rejects_stale_revision`, `library_bad_file_is_not_overwritten`. Assert reopened data equals saved data; failing disk write leaves prior revision and disk bytes unchanged; revision mismatch returns conflict; malformed JSON/unknown version load errors leave original bytes intact and block mutations.
- [ ] Add load-validation cases for duplicate IDs/paths, dangling project IDs, invalid preferences and invalid schema; do not silently salvage data. Test missing file loads empty without importing history.
- [ ] Run `cd src-tauri && cargo test --test library_store`; verify RED for persistence cases.
- [ ] Implement load/validate, mutation under lock, same-directory temporary write and atomic replacement; increment revision only on successfully committed mutations. Use an injectable writer in unit tests to reliably simulate write/rename failure, not OS permission assumptions. Do not fall back to an empty file on I/O/parse/version errors.
- [ ] Register commands and state with app_data_dir isolation; load is retryable from UI. Enforce expected revision on every mutation and return the full confirmed snapshot. State/library mutex covers the whole write, not only snapshot construction.
- [ ] Run focused Rust tests and `cargo clippy --all-targets -- -D warnings`; expect exit0. Verify command code cannot write media, history, transcript or summary paths.

### Task 3: Reactive store and unified ingestion without playback coupling

**Files:** new library types/service/store and mediaIngestion.ts; existing App.vue, useKeyboardShortcuts.ts, playlistStore.ts; tests/unit/libraryStore.test.ts, openMedia.test.ts, keyboardShortcuts.test.ts.

**Interfaces:** Consume task2 IPC; implement shared frontend contracts. Coordinator `ingestMedia(paths:string[], projectId:string|null = null):Promise<void>` registers the entire batch, then calls the existing queue add behavior once. `playLibraryRecording(path:string):Promise<void>` ensures queue membership and plays exactly that path without registering it again. Introduce playlist `ensureFiles(paths:string[]):void` (no autoplay), retain addFiles behavior using this helper.

- [ ] Write failing store tests with deferred IPC: `startup_import_waits_for_saved_library`, `queued_edits_use_latest_revision`, `failed_edit_keeps_confirmed_snapshot`, `conflict_reloads_without_replaying_edit`. Assert no load/seek during initialization; no saved metadata disappears; load failure settles with writesEnabled=false; conflict returns false and retains editor draft for retry.
- [ ] Write failing ingress tests: duplicate open retains metadata; global ingestion uses null group; project-scoped ingestion assigns only new paths; folder scan and file picker pass through the same coordinator; load/storage failure still allows ordinary playback with visible unsaved-library warning.
- [ ] Run `npx vitest run tests/unit/libraryStore.test.ts tests/unit/openMedia.test.ts tests/unit/keyboardShortcuts.test.ts`; record RED.
- [ ] Implement serialized store apply operations, initialization promise and error handling. Generate proposed entry/project UUIDs at the mutation boundary, retaining them for explicit retries. Keep errors visible to callers; never optimistically claim persistence.
- [ ] Route each existing open path through ingestMedia. Inspect native/frontend drop handling before changing it; if no shared frontend drop route exists, add one Tauri drop listener with cleanup through mediaIngestion, avoiding duplicate native handling. Queue events received before initialization rather than dropping them. Explicit selection uses ensureFiles then path lookup so insertion cannot autoplay twice.
- [ ] Run focused tests; expect PASS. Assert no import is triggered by duration/history/player-progress updates and no playback history auto-import happens at startup.

### Task 4: Searchable grouped browser and editing interactions

**Files:** utils/libraryView.ts; five components/library files from ownership map; ReadingSidebar.vue; locales/en.ts + th.ts; tests/unit/libraryView.test.ts and libraryBrowser.test.ts.

**Interfaces:** `recordingName(recording:LibraryRecording):string`; `visibleLibrary(data:LibraryData, query:string, recentFiles:HistoryItem[]):LibraryView` where LibraryView contains `pinned:LibraryRecording[]`, `groups:{id:string,name:string,collapsed:boolean,matching:number,total:number,items:LibraryRecording[]}[]`. Store displayName delegates to recordingName or basename fallback. Components emit typed mutations through store.apply, not IPC directly.

- [ ] Write failing view tests: search renamed and original names case-insensitively; collapsed matches appear without changing preferences; clearing search restores collapse; pinned shortcut does not inflate total; sort added/played descending, name via Intl.Collator for active locale, stable ID tie-break; never mutate queue or input arrays. Include 1,000 records and 50 groups; validate exact result counts, not flaky wall-clock thresholds.
- [ ] Write failing mounted tests: Enter saves trimmed name, Escape cancels, failed save keeps draft; 120/121 scalar-value limits match Rust; long duplicate basenames remain distinguishable; focus returns to invoking menu after close; project deletion confirmation preserves files; removal copy explicitly states originals/transcripts/summaries are retained.
- [ ] Run `npx vitest run tests/unit/libraryView.test.ts tests/unit/libraryBrowser.test.ts`; record RED.
- [ ] Implement compact two-line rows using existing theme/font/icon styles. Add search, sort, pinned shortcuts, group counts/collapse, create/rename/delete project, item menus, move-group picker including Uncategorized and reset-name action. Menus/buttons remain keyboard discoverable, including when not hovered. No-results, empty-library, saving, load-error/retry and unavailable-file states are distinct.
- [ ] Keep query/menu/editor state transient. Persist sort/collapse via confirmed mutations only; search does not write preferences. Keep Open file/folder and add project-scoped import actions; deleting groups/removing records requires affirmative confirmation.
- [ ] Run focused UI tests plus `npm run typecheck` and `npm run lint`; expect PASS. Record screenshots during task6, not an unverified visual-completion claim here.

### Task 5: Display-name integration and playback regression protection

**Files:** PlaylistItem.vue, PlaylistDrawer.vue, TopBar.vue, active header/player dock bindings identified in file map; playlistStore.ts and mediaIngestion.ts if required; tests/unit/libraryPlayback.test.ts, existing readingWorkspace.test.ts and playbackHistory.test.ts.

**Interfaces:** Consume libraryStore.displayName and playLibraryRecording. UI active state compares original path to player media path, never filtered array index. Library browsing never invokes playlist reorder/clear.

- [ ] Write failing `library_browsing_does_not_change_queue`, `removing_active_library_record_keeps_playing`, `renames_only_change_visible_labels`. Assert queue paths/order/current item, player load-call count, resume keys and summary/transcript load arguments are unchanged after rename/move/pin/filter; selecting a filtered row loads its original path once.
- [ ] Add `missing_file_remains_in_library`: failed access shows unavailable/retry/remove and retains record; next successful retry clears transient error. Validate actual existence through a narrow backend read-only check command if player errors cannot reliably identify the path; do not treat a generic decoder error as file-not-found.
- [ ] Add `queue_clear_keeps_library` and `removed_record_is_not_readded_by_progress`; mock playback updates and assert zero add_recordings calls. Assert queue labels fall back to basename after library removal.
- [ ] Run `npx vitest run tests/unit/libraryPlayback.test.ts tests/unit/readingWorkspace.test.ts tests/unit/playbackHistory.test.ts`; record RED.
- [ ] Implement shared names on identified display surfaces, preserve raw filename/path in media info and AI inputs, label queue versus library clearly. Any missing-file check must be read-only and must not canonicalize/change stored paths.
- [ ] Rerun focused tests and all frontend tests; expect PASS. Review diff specifically for accidental changes to summary/chat/cache identity.

### Task 6: Isolated restart, visual QA and full verification

**Files:** PLAN.md sections16/17 and applicable source-of-truth note; design-qa.md; test fixtures live only in a unique temporary directory. No production configuration edits.

**Interfaces:** Uses completed tasks1–5. No new product behavior in this task.

- [ ] Run `cargo fmt` then `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` in src-tauri; run `npm run typecheck`, `npm run lint`, `npx vitest run` at root. Record exit codes and counts; do not rebuild a running test executable concurrently. Diagnose any failures before claiming completion.
- [ ] Build a separately identified test app through a temporary Tauri configuration using the existing Preview convention; verify its app_data_dir is isolated before opening fixtures. Do not replace /Applications/Velo.app or close existing previews.
- [ ] Create synthetic fixture recordings/cache files in the isolated directory, record names/paths/hashes. Set display names, groups, pins, sort/collapse; close only this test instance and reopen it. Verify metadata survives, no autoplay, and media/cache hashes are identical. Test library corruption on a disposable copy and recover by restoring that copy only.
- [ ] Inspect actual light/dark/narrow UI, long Thai/English names, keyboard menus and rename, search across collapsed groups, missing media and 1,000-record browsing. Capture screenshots and report responsiveness observations. If lag is measurable, optimize rendering without changing data ownership or adding a dependency; rerun affected tests.
- [ ] Update PLAN.md to describe persistent Rust library versus transient Pinia queue and actual app_data_dir; append evidence and any remaining limitations to design-qa.md. Do not claim cloud summary tests were run for this unrelated feature.
- [ ] Review all task changes against the approved spec, then request one read-only final reviewer per the Native workflow if the available review mechanism permits it. No reviewer may edit, push or release. Resolve substantive findings and rerun affected gates.
- [ ] Report what works, checks actually completed, Preview isolation and unchanged original files/data. No push, release, tag or production migration.

## Plan self-review

- Spec coverage: persisted metadata and recovery (tasks1–3), complete sidebar actions (task4), display and queue separation (task5), restart/visual/scale checks (task6).
- Review Focus cases each have named tests in their owning tasks. Source paths remain identity everywhere; no stale index is used to select filtered rows.
- Shared command/type/store names are consistent across tasks. Explicitly reject malformed files rather than borrowing HistoryStore's silent-default behavior.
- Current Native method is preserved. This plan awaits the user's review before implementation; it does not authorize a new worktree, app restart or unrelated summary changes.
