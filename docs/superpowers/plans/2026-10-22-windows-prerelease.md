# Windows Preview Installer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for task-by-task execution. Native execution continues in this session; if that skill is unavailable, follow these checks directly with TDD and verification-before-completion. No parallel implementation agents.

**Goal:** Push the user's accumulated work, build Windows x64 installers in GitHub Actions, and publish a testing-only prerelease without changing the stable updater channel.

**Architecture:** Preserve the existing macOS release path. Add pinned Windows dependencies and an explicit prerelease path that builds installers, publishes only after checks pass, and never writes latest.json or marks a release latest. Preview builds use a separate application identifier and no updater endpoints.

**Tech Stack:** Tauri v2, Rust/MSVC, Vue, NSIS, PowerShell, GitHub Actions.

**Spec:** Approved conversation scope on 2026-10-22: include all latest work, add Windows build, push, and create a prerelease; never set Latest or distribute automatic updates. Existing architecture: PLAN.md sections 11, 24 and 25. This plan supersedes earlier no-push restrictions only for this approved delivery.

## Global Constraints

- Proposed version: `0.9.0-beta.1`; verify tag does not exist before use.
- Windows target: `x86_64-pc-windows-msvc`, NSIS per-user installer. No MSI or ARM64 in this delivery.
- Preview identifier: `com.velo.preview`; display name `Velo Preview`. Do not migrate production settings, library, caches or credentials into it.
- Empty updater endpoints and disabled updater artifacts in preview configuration. No signing-key access is needed for this prerelease path.
- Release must remain `prerelease=true`, `make_latest=false`; never upload `latest.json`.
- Do not claim Windows hardware/UI acceptance based on compilation or hosted-runner smoke checks. Explicitly list untested GPU, microphone and Windows-version cases in Thai release notes.
- Existing user work must be preserved. Review tracked and untracked files before staging; exclude credentials, private recordings, generated logs and private acceptance artifacts.
- Use pinned dependency URLs and SHA-256, not floating latest download links. Preserve dependency provenance/licenses and corresponding source information with the installer.
- No live summary API calls for packaging tests.
- Follow repository release ordering: feature PR/checks/merge, synchronized version and Thai changelog PR/checks/merge, then tag. Do not bypass branch protections.

## Review Focus

- Missing native DLLs: installed app must launch with no mpv installation or development PATH.
- Wrong release routing: beta tags and retries must never reach stable publish or updater-manifest jobs.
- Existing production installation: Preview must not overwrite production files, settings or credential namespace.
- Portable filenames and permissions: Thai names, spaces and standard-user install must work.
- Partial build/retry: publish only complete validated assets; do not replace assets of an already published prerelease.

### Task 1: Version and prerelease routing

**Files:** `scripts/version.mjs`, `.github/workflows/release.yml`, new `tests/release/prerelease.check.mjs`, preview config under `src-tauri/`.

**Interfaces:** Version script accepts strict SemVer prerelease input; release routing derives preview status from validated version/tag, not arbitrary shell fragments.

- [ ] Write executable tests using temporary version/changelog fixtures: `0.9.0-beta.1` updates every version source consistently, mismatches and malformed versions fail without partial edits, existing stable version behavior remains valid.
- [ ] Run new tests and observe failure before implementation.
- [ ] Extend version parsing and changelog linking for prerelease versions. Add pure tested release-policy logic returning prerelease/latest/updater decisions for stable and beta versions; wire workflows to that logic.
- [ ] Add isolated Preview config with no updater endpoints; validate against installed Tauri config schema and check effective merged config before build.
- [ ] Test beta policy produces prerelease=true, latest=false and updater=false; stable path retains current policy. Test malformed input and published-release retry rejection.
- [ ] Run `node --test tests/release/*.check.mjs` and `npm run version:check`.

### Task 2: Pinned Windows build and packaging

**Files:** new `scripts/setup-windows.ps1`, pinned dependency metadata, `src-tauri/build.rs`, Windows bundle config, dependency notice.

**Interfaces:** Setup exports `VELO_MPV_LIB_DIR` for build.rs and a deterministic runtime directory for installer resources and test PATH. Generated dependencies remain ignored.

- [ ] Pin upstream `shinchiro/mpv-winbuild-cmake` release `20261021`, asset `mpv-dev-x86_64-20261021-git-e471496c56.7z`, SHA-256 `efc719eff3dc5954b4a5efc800632808bcec01cc9034940dd14e5eb7fe9ea633`; verify availability again at execution.
- [ ] Inspect archive layout, export definitions, DLL dependency closure, license files and source provenance before implementing extraction rules. Do not assume FFmpeg DLLs are separate or statically included.
- [ ] Implement fail-fast checksum verification, extraction to a repository-local ignored directory and MSVC import-library generation. If no mpv.def exists, derive exports from the inspected DLL with the MSVC tooling and verify the generated import library.
- [ ] Add Windows-specific link search handling while preserving macOS pkg-config behavior. Fail with actionable diagnostics when explicit Windows setup is invalid.
- [ ] Bundle runtime DLLs next to executable; include notices and corresponding-source references. Use WebView2 bootstrapper installation and document first-install network requirements.
- [ ] On Windows, test wrong checksum and missing archive exports fail before build; run setup successfully, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and NSIS bundle build.

### Task 3: Actions verification and installer smoke test

**Files:** `.github/workflows/ci.yml`, preview build workflow or explicit preview jobs in release workflow, `scripts/test-windows-installer.ps1`.

**Interfaces:** Build jobs produce uniquely named installer artifacts plus checksum/provenance files. Publishing consumes only successful build outputs from the selected commit.

- [ ] Set up MSVC x64, Rust, Node, CMake and required whisper.cpp tooling on a Windows runner. Initialize libmpv before compile/link/tests.
- [ ] Run frontend typecheck/lint/tests, Rust fmt/clippy/tests, release-script tests, and Windows installer build. Keep opt-in live tests disabled.
- [ ] Install NSIS output into an isolated temporary path as a normal user, remove development dependency paths from the launched process environment, verify required packaged files, and assert installed process stays alive and exposes its window. Clean up only that test installation.
- [ ] Upload installer/checksums/log artifacts for debugging; do not publish a prerelease if mandatory checks fail. A process smoke test is not proof of video or microphone functionality.
- [ ] Keep macOS build coverage and produce isolated macOS Preview artifacts if included in the prerelease. Reuse dependency-repair and self-contained-bundle validation without publishing updater archives.

### Task 4: Review accumulated work, push and publish

**Files:** existing modified/untracked product files, `CHANGELOG.md`, `PLAN.md`, `CONTRIBUTING.md`, release notes and synchronized version sources.

- [ ] Review staged diff and secret/private-data scan findings before committing. Resolve stale changelog claims about extractive-only summaries against the actual direct-summary code. Mark historical evidence experiments as historical rather than current acceptance claims.
- [ ] Run all local checks: `cargo fmt`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run typecheck`, `npm run lint`, `npx vitest run`, release-script tests and macOS app bundle build.
- [ ] Commit accumulated work plus packaging changes on the existing branch, push, open/attach PR and wait for Actions. Investigate every failure from logs; add regression tests for code bugs before fixing them.
- [ ] Merge only after mandatory checks pass. Bump proposed beta version using the tested version script, prepare accurate Thai release notes, PR/merge the version change, then tag the merged commit.
- [ ] Publish via the prerelease-only path after asset verification. Verify GitHub reports isPrerelease=true, stable latest remains v0.8.0 (or the independently current stable release), and no latest.json is attached or changed.
- [ ] Hand off installer URL, build run URL and SHA-256. State that physical Windows GPU/audio/microphone acceptance remains pending unless a real Windows test environment becomes available.

## Manual Windows acceptance before stable promotion (not a prerelease claim)

- [ ] Windows 11 x64 clean install, no developer tooling or mpv; separately test Windows 10 before advertising support.
- [ ] MP4/MP3/WAV, Thai filenames, spaces, seeking, pause, fullscreen, resizing, 100–200% DPI and multiple displays.
- [ ] Native video stays behind UI; controls receive pointer/keyboard input; player panel opens/closes correctly.
- [ ] Whisper real model, cancellation, microphone permissions/disconnect and Windows Credential Manager persistence with disposable test credentials.
- [ ] Library/projects, default font and chosen preferences survive relaunch/upgrade. Uninstall does not silently delete user data.
- [ ] Stable updater work remains a separate authorized task after these checks; no automatic-update delivery in this prerelease.
