# Contributing to Velo

Thank you for your interest in contributing to Velo! We welcome contributions from the open-source community.

---

## Development Prerequisites

* **Node.js**: v18+ (Node 20+ recommended)
* **Rust**: 1.75+ (via [rustup](https://rustup.rs))
* **libmpv & pkg-config**:
  * **macOS**: `brew install mpv pkg-config`
  * **Windows**: Download prebuilt `libmpv-2.dll` and developer headers or install via Chocolatey/vcpkg.

---

## Getting Started

1. **Clone the repository**:
   ```bash
   git clone https://github.com/emozerorise/velo.git
   cd velo
   ```

2. **Install frontend dependencies**:
   ```bash
   npm install
   ```

3. **Start development mode**:
   ```bash
   npm run tauri dev
   ```

---

## Project Structure

* `src/`: Vue 3 + TypeScript frontend application.
  * `components/`: UI overlay components (player dock, timeline, modal dialogs, playlist drawer).
  * `stores/`: Pinia state management (`playerStore`, `playlistStore`, `settingsStore`).
  * `services/`: Tauri IPC communication bridge.
* `src-tauri/`: Rust backend and native integration.
  * `src/player/`: Safe encapsulation of `libmpv`, background event loop, and property observation.
  * `src/platform/`: Platform-specific native view hierarchies (`NSView` on macOS, `HWND` on Windows).
  * `src/storage/`: Atomic JSON settings and history storage.

---

## Code Quality & Testing

Before submitting a pull request, please ensure all checks pass:

```bash
# Frontend
npm run lint
npm run typecheck
npm run test

# Rust
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

`npm run lint:fix` applies what ESLint can fix on its own, and
`npm run format` runs Prettier. Formatting rules are left to Prettier, so
ESLint only reports things it can reason about.

One rule is worth knowing before it surprises you:
`@typescript-eslint/no-floating-promises`. An `invoke` call that is never
awaited and never catches will fail silently, which looks exactly like a
button that does nothing. Actions meant to be fire-and-forget therefore
return `void` and report their own errors -- see `dispatch()` in
`src/stores/playerStore.ts`. Follow that shape rather than adding `void`
at the call site, unless the promise genuinely cannot reject.

Note that `cargo test` runs on macOS in CI only: the Windows runner has no
libmpv to link against, so it is limited to `cargo fmt` and `cargo clippy`.

---

## Pull Request Guidelines

1. Create a feature branch with a descriptive name (`feat/my-feature` or `fix/issue-id`).
2. Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
3. Ensure no decoded video frames cross the Tauri IPC barrier.
4. Keep platform-specific code cleanly isolated under `platform/`.
