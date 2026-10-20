# In-app updates

The user approved automatic checks with explicit in-app installation, followed by push and release. Execute in this session. Ship as 0.6.0; users on 0.5.0 install this first updater-enabled version manually once.

## Behaviour

- Check three seconds after settings finish loading, then hourly while open. Automatic requests are limited to once per 24 hours, across restarts; manual checks bypass that limit. Users can disable automatic checks and skip a version.
- Finding an update only shows an unobtrusive notice. Downloading starts after a click. Show release notes as plain text and download progress, including downloads without Content-Length. Dismissal lasts for the session; skipped versions stay hidden from automatic checks but remain available to manual checks.
- Download and installation are separate operations. Installation and restart are blocked during transcription, model downloads, or summary jobs. Recheck the guard after downloading; a job may have started meanwhile. A downloaded update remains available to install later.
- Installation presents a blocking UI, and keyboard shortcuts cannot start other work during installation. Restart is a separate explicit click, guarded again. Persist the current playback position before restarting, respecting the remember-position preference. Never restart automatically.
- Network/check errors do not interrupt playback. Settings shows an actionable retry. Duplicate checks/downloads are prevented, failed downloads are retryable, and failed installation must not be shown as success.

## Architecture

Use the official Tauri updater and process plugins, so all update HTTP and signature verification stays in Rust. A small frontend service owns the native API boundary. A Pinia update store owns the update handle, state, preferences, scheduling and busy guards. Add defaulted `updates` settings so older settings files retain their other preferences. Settings gets an Updates tab and App mounts a persistent update notice.

The trusted public key and HTTPS GitHub Releases endpoint are compiled into the app. Store the signing private key outside the repository, mode 0600, and in a repository Actions Secret. Never print or commit the private key. Updater signatures are separate from Apple code signing; keep the project's existing ad-hoc macOS signing.

## Release protocol

Build and repair each architecture's bundle exactly as before, including bundled libmpv and self-contained verification. Only then create `.app.tar.gz` and sign those exact bytes. Upload DMG, archive and signature to a draft release. A final job requires both architectures, validates their archive names/signatures, creates `latest.json` for `darwin-aarch64` and `darwin-x86_64`, uploads it, and publishes the release. A failed architecture must leave the prior complete release as latest. Serialize release runs; do not expose a partially-built release through the updater endpoint.

## Validation

Vitest covers throttling across launches, opt-out, manual override, skipped versions, duplicate requests, unknown-length progress, failed download retry, busy-at-start and busy-after-download, install/restart failures and history flush before restart. Rust tests cover backwards-compatible settings defaults. Node tests execute the manifest generator against controlled files and reject missing/empty signatures or archives. Run Rust fmt/clippy/tests, frontend lint/typecheck/tests, release-script tests and an Apple Silicon bundle build. Check actual signed archives and the published manifest before reporting release completion.
