# Project library and display names

Status: proposed specification for user review. No implementation authorized by
this document alone. No push or release.

## Intent

Replace the reading workspace's flat file list with a searchable, persistent
library organized into project groups. Users can give recordings meaningful
display names without renaming, moving or copying their media. This serves
meeting/work recordings, not multiple saved playback playlists.

The user approved display-name editing, project grouping and persistence across
launches. Existing light/dark styling, Noto Sans Thai, accent color and icon
library remain. All new UI text is bilingual through en.ts and th.ts.

## Scope and interaction

- Sidebar title becomes Library / คลังไฟล์. Keep Open file and Open folder.
- Add search over display name and original basename, a New project action,
  and a sort menu: recently added (default), recently played, name.
- Show collapsible project groups with counts, plus an Uncategorized group.
  A recording belongs to zero or one project; nested projects are out of scope.
- A pinned section provides shortcuts to pinned recordings. The same recording
  remains in its project; the shortcut is not a duplicate library record.
- Rows use a compact two-line layout: display name, then duration and original
  basename when renamed. Missing duration is omitted, not shown as zero.
  Long names truncate visually but remain available through accessible labels
  and tooltips. A clear active marker identifies the playing file.
- Each row has a keyboard-accessible menu: Rename display name, Move to project,
  Pin/unpin, Reset display name (when set), Remove from library.
- Rename opens a focused input: Enter saves, Escape cancels. Trim outer whitespace;
  reject empty names and names over 120 Unicode characters. Reset is explicit.
  Duplicate recording display names are allowed; original filename distinguishes
  them. Never change the media path or extension.
- Projects support create, rename and delete. Project names use the same length
  rules; reject case-insensitive duplicate names after trimming. Deleting a
  project requires confirmation and moves its recordings to Uncategorized.
- Removing a recording requires confirmation explicitly saying the original file,
  transcript and summary are retained. No bulk removal or filesystem deletion.
- Search examines all projects, including collapsed groups, and reveals matching
  rows without overwriting saved collapse preferences. Show matching/total counts
  and a useful no-results state. Clearing search restores the previous view.
- Search and sorting affect browsing only, never the running playback sequence.
  Adding from inside a project assigns new records to that project; global file
  open, drag/drop and folder ingestion default to Uncategorized. Reopening an
  existing path preserves its name, pin and project rather than moving it.

## Playback boundary

The existing Pinia playlist is a transient playback queue, not the persistent
library. Selecting a library recording ensures it is in the queue and plays it
by stable identity/path. Merely filtering, sorting, grouping or restoring the
library must not load media, seek, autoplay or change next/previous behavior.

Removing a library record does not interrupt current playback or delete the
queue entry. Existing queue clear/remove operations do not erase library data.
The queue drawer remains a playback-specific surface and uses display names
resolved from the library, falling back to the basename. This distinction must
be visible in labels. Refreshing a duration or playback position is not a library
insertion, so playing a removed queue item does not silently re-add it.

Display-name resolution is shared by the reading sidebar, active-media header,
player dock and queue. Transcript/summary caches, chat context and history/resume
identity continue to use the original path. Display names never become AI source
material or alter existing prompt behavior.

## Persistence and ownership

Add a Rust LibraryStore with a versioned library.json in the app's existing
app_data_dir, separate from settings.json, history.json and transcript caches.
Do not hard-code the historical PLAN.md storage-path examples. The production
and Preview bundle identifiers keep their respective app-data isolation.

Version 1 contains:

- projects: stable ID, name, created timestamp;
- recordings: stable ID, exact original absolute path, optional display name,
  optional project ID, pin flag, added timestamp, optional known duration;
- preferences: collapsed project IDs and sort order;
- revision: monotonically increasing integer for stale-write detection.

Use exact-path deduplication consistent with existing ingestion. Do not change
paths via canonicalization or merge symlink aliases in this iteration. Derive
original basename from path. Last-played and resume data remain in HistoryStore.

Expose library load and typed mutation commands through commands/library.rs.
Rust serializes mutations under a lock, validates IDs/references/names, writes
temporary JSON in the same directory, then atomically replaces library.json.
Publish the new in-memory revision only after disk success. The frontend mirrors
confirmed snapshots and serializes its edits; stale revisions cause reload and
a visible retry message, not silent overwrite. Editors keep drafts on failure.

Library initialization completes before ingestion; early open/drop events are
queued until load completes. Missing library.json starts an empty library. Do not
automatically import all playback history: it could restore deliberately removed
or unrelated files. Existing transient queue records can be explicitly added by
the user; no running app will be restarted to migrate its memory.

Malformed/unreadable JSON or an unsupported version must not be replaced with an
empty library. Preserve the file, show an error, disable library writes and offer
retry; ordinary file playback remains usable. Missing media remains listed and
is marked unavailable after a failed access, with retry/removal actions. Do not
silently remove records or relocate files. External-file relinking is deferred.

## Implementation boundaries

Add library types/service/Pinia store and focused sidebar row/group/menu/editor
components. Integrate with ReadingSidebar and file/folder/drop ingress, leaving
summary inference and native rendering untouched. Add Rust storage/commands and
register managed state in app initialization. Update PLAN.md to distinguish the
actual frontend queue from the persistent Rust library; the current section16
describes a Rust queue that the existing implementation does not have.

Use local component state for search, menus and drafts. Persist only agreed
library metadata/preferences. No API calls, cloud sync, dependencies for a
database, tags, nested folders, thumbnails, drag-reordering or multiple playlists
are needed for this scope.

## Verification and acceptance

1. Rename, group, pin, sort and collapse survive a real app restart using isolated
   test data. Media filename/path/bytes and existing summary/transcript caches
   remain unchanged. Do not close the user's running app without asking.
2. Importing the same path twice preserves metadata and creates one record.
   Open-file, folder and drag/drop entry points obey the same rules.
3. Project deletion preserves recordings in Uncategorized. Library removal
   preserves disk files/cache and does not stop the active recording.
4. Filtering/sorting/pinning/project moves do not select the wrong queue item or
   change playback order, next/previous, resume positions or active-media identity.
5. Unit/integration tests cover persistence round trips, invalid references,
   duplicates, stale revisions, failed writes, corrupt/unsupported files and
   startup ingestion races. UI tests cover editor cancel/save/error, search across
   collapsed projects, name fallback, empty/missing-file states and keyboard use.
6. Inspect light/dark and narrow-sidebar layouts with long Thai/English names;
   exercise 1,000 synthetic records for scrolling and search responsiveness.
7. Run cargo fmt/check, cargo clippy --all-targets -- -D warnings, cargo test,
   npm run typecheck, npm run lint and npx vitest run. Report actual outcomes.

## Next gate

User review of this specification precedes a Native implementation plan. Preserve
all existing uncommitted work. No push, release or production-data migration.
