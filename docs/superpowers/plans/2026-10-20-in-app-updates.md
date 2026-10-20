# In-app Updates Implementation Plan

**Goal:** Ship the approved in-app update flow and a complete signed 0.6.0 release.

**Architecture:** Official Tauri updater/process plugins behind a frontend service; a Pinia state machine handles checks, explicit installation and restart. A final release job publishes the update manifest after both repaired macOS bundles are signed.

**Tech Stack:** Vue 3, Pinia, Tauri v2, Vitest, Rust, Node's built-in test runner, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-20-in-app-updates-design.md`

## Constraints and review focus

Execute inline as requested. Preserve existing playback, transcription and summary interfaces. Keep HTTP in Rust, all UI bilingual, and signing keys out of Git and logs. Test jobs starting during download, stale check timestamps after clock changes, missing Content-Length, retry after native errors, and partial release builds.

## Task 1: Update state and persistence

Files: `src/services/updateService.ts`, `src/stores/updateStore.ts`, `src/types/settings.ts`, `src/stores/settingsStore.ts`, `src-tauri/src/storage/settings_store.rs`, `tests/unit/updates.test.ts`.

- [x] Write failing tests for `checkForUpdates(manual = false): Promise<void>`, `downloadAndInstall(): Promise<void>`, `restart(): Promise<void>`, and `skipVersion(): Promise<void>`. Assert checks stop within 24 hours or when disabled, manual requests bypass both, skipped versions remain hidden automatically, one in-flight request is made, native errors preserve a retry path, and no restart occurs without an explicit call.
- [x] Run `npx vitest run tests/unit/updates.test.ts` and confirm the missing store fails.
- [x] Implement `updateService` using native `check`, `Update.download`, `Update.install`, `Update.close`, `getVersion`, and `relaunch`. Implement the store with phases idle/checking/available/downloading/downloaded/installing/ready/error, post-download busy guards, session dismissal, and a separate restart action that awaits history persistence.
- [x] Default `updates` to `{ auto_check: true, last_checked_at: 0, skipped_version: '' }` in both languages. Add a Rust legacy-settings test before implementation, run it red then green.
- [x] Run the focused frontend and Rust tests.

## Task 2: UI and app lifecycle

Files: `src/components/updates/UpdateSection.vue`, `src/components/updates/UpdateNotice.vue`, `src/components/settings/SettingsModal.vue`, `src/App.vue`, `src/locales/en.ts`, `src/locales/th.ts`, `src/composables/useKeyboardShortcuts.ts`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`.

- [x] Register updater/process plugins and their narrowly scoped capabilities.
- [x] Mount a notice outside auto-hiding player controls and add an Updates tab. Render current version, plain-text release notes, errors, progress, skip/later/check/install/restart actions and the opt-out checkbox.
- [x] Start delayed checks after settings load, repeat hourly and stop timers on unmount. Block keyboard actions while installing and cover the app with an installation overlay.
- [x] Run `npm run typecheck`, `npm run lint` and `npx vitest run`.

## Task 3: Signed release packaging

Files: `.github/workflows/release.yml`, `scripts/update-manifest.mjs`, `tests/release/update-manifest.test.mjs`, `src-tauri/tauri.conf.json`, `README.md`, `CHANGELOG.md`, `.claude/skills/release/SKILL.md`.

- [x] Write manifest-generator tests using real temporary fixture archives and signature files. Require exactly the two macOS platforms, correct HTTPS asset URLs and version, and reject absent/blank inputs. Run `node --test tests/release/*.check.mjs` red.
- [x] Implement the generator, signing step after dylib repair, draft release uploads and a final manifest/publish job gated on both builds. Generate a protected updater key if none exists, provision the Actions Secret without logging it, and embed only its public key.
- [x] Verify actual archive signatures locally, and run the manifest tests green. Document key location/backup requirements and manual installation of the first updater version. The real repaired bundle passed signature verification; changing one byte was rejected. Release CI runs the same verification for each architecture.
- [x] Run fmt/clippy/cargo test, typecheck/lint/Vitest, manifest tests and a local Apple Silicon bundle build. Push/merge the feature PR after CI passes.
- [ ] Bump all version sources with `npm run release 0.6.0`, push/merge its release PR, then tag merged main. Wait for both architectures and manifest publication; verify assets, endpoint, tag commit, and clean local main.

Final smoke-test finding: manual update controls could save defaults before
startup finished loading settings. Cancelled the first unpublished, asset-free
draft build and added a regression test and readiness guard before retrying.
