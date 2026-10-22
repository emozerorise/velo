# Thai Reading Font Implementation Plan

> **For agentic workers:** Execute this bounded, user-approved change in the current session. Preserve existing uncommitted work; do not push or release.

**Goal:** Make Thai summary, transcript, and chat text easier to read offline.

**Architecture:** Bundle Noto Sans Thai and use it as the shared sans family. Apply a shared 16px/1.8 reading style without changing content, summary behavior, or monospace timestamps/code.

**Tech Stack:** Vue 3, Tailwind CSS, Vite, Tauri.

**Spec:** User-approved design in this conversation: looped Noto Sans Thai, bundled offline, body approximately 16px/1.8, headings weight 600, existing time/code font preserved.

## Global Constraints

- No cache, settings, API, Evidence Ledger, version, push, or release changes.
- Keep both UI languages and light/dark themes working.
- Include the complete upstream SIL Open Font License with the font.

## Review Focus

- Thai stacked marks must remain visible in multiline text and inputs.
- Narrow transcript panels must wrap without horizontal clipping.
- English mixed with Thai must retain clear hierarchy.
- Timestamp and code glyphs must remain monospace.
- A fresh session must load the font locally without a network font service.

### Task 1: Local Thai reading typography

**Files:** `src/assets/fonts/`, `src/assets/styles/main.css`, `tailwind.config.js`, summary/transcript/chat reading components.

**Interfaces:** Existing component props, stores, content, and IPC remain unchanged. A shared `.reading-text` class supplies the body size and line height.

- [x] Render the actual summary/transcript/chat components with synthetic Thai in a local browser fixture; confirm the current font is not the approved bundled family.
- [x] Vendor the official variable font, license, provenance, and SHA-256.
- [x] Add a local `@font-face`, shared sans family, and 16px/1.8 reading style. Remove the previous reading-mode size override. Preserve heading weights and code/time styles.
- [x] Verify the real loaded font, computed size/line height, Thai diacritics, wrapping, themes, and monospace in the browser fixture.
- [x] Run cargo fmt, cargo clippy --all-targets -- -D warnings, cargo test, npm run typecheck, npm run lint, npx vitest run, and a production frontend build.
- [x] Review the diff and confirm original cache/settings hashes are unchanged. Leave the work local and uncommitted for review.
