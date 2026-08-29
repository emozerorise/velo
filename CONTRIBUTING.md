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
npm run typecheck
npm run test

# Rust
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## Pull Request Guidelines

1. Create a feature branch with a descriptive name (`feat/my-feature` or `fix/issue-id`).
2. Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
3. Ensure no decoded video frames cross the Tauri IPC barrier.
4. Keep platform-specific code cleanly isolated under `platform/`.
