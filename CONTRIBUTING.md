# Contributing to Velo

## Windows Preview builds

Use an x64 MSVC developer shell with Rust, Node.js, CMake, LLVM/libclang and
7-Zip and Vulkan SDK 1.4.363.0 installed. Set `VULKAN_SDK` to the SDK root
(with `Lib/vulkan-1.lib` and `Bin/glslc.exe`), then run `./scripts/setup-windows.ps1` before Cargo commands.
This verifies the pinned libmpv checksum and creates the MSVC import library.
Run `npm ci`, then `npm run tauri build -- --target x86_64-pc-windows-msvc --bundles nsis --config src-tauri/tauri.preview.conf.json`.
The Windows Preview workflow runs the checks and installs/launches the package
with a clean PATH. It is not GPU/microphone acceptance on physical hardware.
Windows transcription compiles whisper.cpp with Vulkan GPU compute, including
NVIDIA RTX 50-series support, and retains whisper's CPU fallback. It uses the
application-local Vulkan loader already bundled for mpv; users do not need a
CUDA toolkit. macOS continues to use Metal. See
[the local benchmark procedure](docs/windows-transcription-gpu.md) to compare
both backends with the same recording and speech model.

Preview data and credentials use `com.velo.preview`, not the production ID.
Prereleases have no updater endpoints and never publish `latest.json`.

Thank you for your interest in contributing to Velo! We welcome contributions from the open-source community.

---

## Development Prerequisites

* **Node.js**: v18+ (Node 20+ recommended)
* **Rust**: 1.75+ (via [rustup](https://rustup.rs))
* **libmpv & pkg-config**:
  * **macOS**: `brew install mpv pkg-config`
  * **Windows**: Download prebuilt `libmpv-2.dll` and developer headers or install via Chocolatey/vcpkg.
* **CMake**: required to build whisper.cpp, which is compiled into the binary
  for offline transcription.
  * **macOS**: `brew install cmake`
  * **Windows**: install from [cmake.org](https://cmake.org/download/) or via Chocolatey.

---

## Microphone dictation checks

Dictation requires an installed local Whisper model. The first explicit mic
click may request microphone access; startup and opening chat never do.
macOS needs the packaged usage description (including its localized resources),
not merely the permission granted to a development terminal. On Windows, check
microphone privacy access for desktop apps if WASAPI denies capture.

Audio remains in memory for at most 60 seconds and is transcribed locally.
Review and insert the proposed text before sending it yourself. Playback pauses
before capture and never resumes automatically. Text chat does not require a
speech model. Synthetic adapters verify lifecycle and UI, not real microphone
permission, Bluetooth behavior or recognition quality; these need deliberate
user-assisted testing. Never reset a user's privacy database as a test setup.

For native UI acceptance without a microphone, `--features dictation-qa` is an
explicit, default-off Cargo build variant using `tests/support/dictation_qa.rs`.
It refuses dictation outside the isolated `com.velo.summary-controls-qa` app ID.
Never use this variant for distribution; normal builds use native capture.

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

`cargo test` runs on macOS and in the Windows Preview workflow. The Windows
job prepares the pinned libmpv import library and runtime before linking.

---

## Pull Request Guidelines

1. Create a feature branch with a descriptive name (`feat/my-feature` or `fix/issue-id`).
2. Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
3. Ensure no decoded video frames cross the Tauri IPC barrier.
4. Keep platform-specific code cleanly isolated under `platform/`.
