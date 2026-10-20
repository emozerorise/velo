# Offline Composer Dictation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a person dictate a short chat question locally, review its text, and send it manually.

**Architecture:** A lazy native capture worker owns the microphone and bounded memory buffer; a separate Whisper worker transcribes after recording stops. Commands supply model/engine/player adapters and a shared Whisper lease without making dictation depend on transcript caches. A keyed Vue store proposes draft insertion rather than sending a chat turn.

**Tech Stack:** Rust/Tauri, pinned CPAL/CoreAudio/WASAPI capture, Rubato sinc resampling, existing whisper-rs, Vue/Pinia and macOS AVFoundation permission bindings.

**Spec:** [Approved design](../specs/2026-10-21-reading-tools-chat-design.md).

## Global Constraints

- Plan 3 of 3; depends on [workspace](2026-10-21-summary-workspace.md) visibility/focus and [chat](2026-10-21-content-chat.md) draft/lease interfaces. Await plan review/execution-method selection.
- One explicit mic click requests permission; no capture/permission on startup, host enumeration or chat opening. No camera or system-audio permission.
- Record at most 60 seconds; normalize to 16 kHz mono f32; audio stays in memory and never reaches webview, disk cache, logs or cloud.
- Reuse installed Whisper; no silent download or second concurrent engine. Ordinary text chat remains usable without mic/model.
- Stop/discard on hiding chat, transcript/file changes, window blur after permission dialog resolution, window close or cancellation. Late results cannot edit another file.
- Pause Velo media before actual capture and do not auto-resume, including error/cancel. A denied permission before capture does not pause playback.
- Returned text is a proposed insertion. Insert appends to the current editable draft; Discard preserves it. Send remains a separate explicit action.
- Preserve deployment targets (bundle currently macOS 11.0; supported macOS policy 12+) and Windows 10; do not solve dependency failures by raising OS minimums. No signing/account requirement.
- UI strings bilingual; English source/comments/commits, Thai changelog. No version bump, PR, push, release or installed-app replacement.

## Review Focus

1. Permission granted after the user cancelled or changed files: no capture starts (Task 2).
2. Bluetooth/default device changes, unsupported formats and nonfinite samples: recoverable error or finite valid PCM, no endless recording (Task 1).
3. Stop/timeout/close race while a callback or Whisper is active: exactly one terminal result, resource released after work ends (Task 2).
4. User edits the composer or switches file before dictation returns: append only by deliberate review; no draft overwrite or auto-Send (Task 3).
5. Fresh packaged install versus a dev terminal with microphone permission: bundle declaration/OS consent really works, not inferred from dev tests (Task 4).

---

## File Structure and Dependency Decisions

- Create `src-tauri/src/dictation/{mod,types,capture,audio,permission}.rs`: lifecycle, bounded capture, pure conversion/resampling, permission abstraction.
- Create `src-tauri/src/commands/dictation.rs`; register commands/state in commands/mod and lib, cancellation on window close in lib.
- Add explicit model-path resolution/engine adapter in `commands/dictation.rs`, using `transcript::model`/`engine` only at this orchestration boundary. No dictation cache writes.
- Modify Cargo.toml/Cargo.lock, tauri.conf.json; create `src-tauri/Info.plist`, `src-tauri/resources/en.lproj/InfoPlist.strings`, `th.lproj/InfoPlist.strings` and wire bundle file mapping.
- Create `src/types/dictation.ts`, `src/services/dictationService.ts`, `src/stores/dictationStore.ts`, `src/components/chat/DictationControl.vue`; use plan 2's composer `dictation` slot.
- Create `src-tauri/tests/dictation_pipeline.rs`, `tests/unit/{dictationStore,dictationControl}.test.ts`; add a bundle-permission manifest check to `tests/release/` and update `design-qa.md`.

Pin `cpal = "=0.16.0"` with default features disabled for macOS/Windows only,
and `rubato = { version = "=0.16.2", default-features = false }`. CPAL 0.18's
documented CoreAudio minimum is macOS 14.2, so do not upgrade to it silently.
Version 0.16 is a compatibility candidate, not proof of a successful Velo
bundle: the unchanged deployment-target bundle build is a mandatory gate.
Use `SincFixedIn<f32>` (one channel, chunk 1024, ratio 16000/input_rate,
relative ratio 1.0, sinc length 256, cutoff 0.95, cubic interpolation,
oversampling 256, BlackmanHarris2 window). Handle filter delay and last partial
block so output length is `round(input_frames * 16000 / input_rate)`.

On macOS pin `objc2-av-foundation = "=0.3.2"`, default features false with
`std`, `AVCaptureDevice`, `AVMediaFormat`, `block2`, and `block2 = "=0.6.2"`.
Use only authorizationStatusForMediaType/requestAccessForMediaType with
AVMediaTypeAudio; no new AVFoundation recording pipeline or camera API. On
Windows lazy CPAL/WASAPI input is attempted only after mic click; translate
OS access denial into privacy-settings guidance, not a promise of a macOS-like
permission dialog. Unsupported targets return Unavailable without loading
platform capture dependencies. Inspect the resolved dependency/license tree
and packaged OS load commands before accepting these pins.

Primary references checked during planning: [CPAL 0.16](https://docs.rs/cpal/0.16.0/cpal/),
[current backend floors](https://github.com/RustAudio/cpal#minimum-supported-versions),
[Rubato 0.16.2](https://docs.rs/rubato/0.16.2/rubato/struct.SincFixedIn.html),
[AVFoundation authorization bindings](https://docs.rs/objc2-av-foundation/0.3.2/objc2_av_foundation/struct.AVCaptureDevice.html#method.requestAccessForMediaType_completionHandler),
[Apple microphone usage declaration](https://developer.apple.com/documentation/BundleResources/Information-Property-List/NSMicrophoneUsageDescription).

### Task 1: Bounded capture and deterministic PCM normalization

**Files:** capture/audio/types modules, target-gated dependencies; audio unit tests and dictation_pipeline harness.

**Interfaces:**

- `CapturedAudio { sample_rate: u32, mono_samples: Vec<f32> }`; default device only, channels 1..8, rates 8000..192000 Hz. Unsupported configurations report a typed error.
- `CaptureBackend: Send + Sync` with `start(&self, cancel: JobCancel) -> Result<Box<dyn CaptureSession>>`; `CaptureSession::stop(self: Box<Self>) -> Result<CapturedAudio>`; dropping a session closes its stream. The session itself stays on its capture worker and need not be Send. A mock backend supplies samples without a device.
- `to_16khz_mono(input: CapturedAudio) -> Result<Vec<f32>>` and `has_speech_energy(samples: &[f32]) -> bool` are pure/testable; no Whisper or HTTP.
- Callback accepts supported CPAL sample formats via its Sample conversion, averages channel samples to mono, clamps finite floats to [-1,1], substitutes zero for nonfinite samples, and writes into preallocated capacity. Capture owns a nonblocking try-lock buffer; contention/overflow marks an error, never blocks or reallocates inside the callback. Resampling runs after stop, outside the callback.
- Maximum mono capture allocation is 192000 * 60 f32 samples (46,080,000 bytes). Actual allocation is device_rate * 60 samples; never grow beyond it. Wall clock and sample count both enforce the limit. A worker owns CPAL Stream so it need not be Send across unrelated runtimes.

- [ ] **Step 1: Write failing tests** `normalizes_signed_unsigned_float_samples`, `stereo_downmix_and_nonfinite_are_bounded`, `bluetooth_8khz_and_44100_and_48000_resample`, `capture_limit_never_reallocates`, `callback_contention_reports_error`, `device_disconnect_releases_stream`, `silence_bypasses_whisper`. Pin one second at 44100/48000 Hz to exactly 16000 finite output samples and a 60-second cap to at most 960000 output samples. Treat audio as silence only when RMS <0.001 AND peak <0.005; reject empty input. These energy checks do not promise rejection of all non-speech noise.
- [ ] **Step 2: Run red:** `cargo test dictation::audio::` and `cargo test --test dictation_pipeline`; expect new contract/normalization assertions to fail. No device capture during tests.
- [ ] **Step 3: Implement the interfaces and pinned adapters.** Delay `cpal::default_host` and device access until start, use supported default input config, keep stream/drop ownership on capture worker, and never fall back to system output/loopback. Add `tokio` time explicitly for timers, not through a transitive feature assumption. Review Cargo.lock/native transitive dependencies; stop for a design update if preserving the existing OS target is impossible.
- [ ] **Step 4: Run green:** those tests, `cargo clippy --all-targets -- -D warnings`, and `cargo tree` inspection. Build an unchanged-target macOS app bundle now, before building the rest on incompatible dependencies. Record the result rather than assuming the pin is sufficient.
- [ ] **Step 5: Commit** with `feat: add bounded native dictation capture and resampling`; inspect that no audio fixtures derived from the user or audio log statements entered the diff.

### Task 2: Permission-safe keyed lifecycle and local engine handoff

**Files:** dictation service/permission module, command module/registration, lib window lifecycle, transcript command adapter; dictation_pipeline tests.

**Interfaces:**

- `MicPermission::{NotDetermined, Granted, Denied, Restricted, Unavailable}`; async `request_microphone_permission() -> Result<MicPermission>` called only by explicit start. Permission completion carries the original session generation; cancelled sessions cannot continue after consent returns.
- `PermissionBackend: Send + Sync` exposes `request(&self, cancel: JobCancel) -> Pin<Box<dyn Future<Output = Result<MicPermission>> + Send>>`; cancellation stops waiting without pretending the OS dialog itself was dismissed. Errors serialize as `permission_denied`, `permission_restricted`, `device_unavailable`, `unsupported_format`, `engine_busy`, `missing_model`, `capture_failed`, `silence`, `transcription_failed`, `cancelled` or `invalid_input`.
- IPC `dictation_start(path: String, session_id: String) -> Result<()>`, `dictation_stop(session_id: String) -> Result<()>`, `dictation_cancel(session_id: String) -> Result<()>`. IDs follow chat's 1..128 ASCII-byte validation; duplicate starts reject and cancel/stop are idempotent for matching terminal sessions.
- Service invoke arguments use `sessionId` (Tauri's default camelCase); serde events use `session_id`. Pin both spellings in service tests.
- Events `velo://dictation-state {path,session_id,phase,elapsed_ms,paused_playback}`, `velo://dictation-ready {path,session_id,text}`, `velo://dictation-error {path,session_id,code,cancelled}`. Phases requesting_permission/recording/transcribing; text stays a proposal, never a chat-ready event.
- `DictationInput` owns path/session/model/language/vocabulary and an engine callback `Fn(&Path, &[f32], &str, &str, &JobCancel) -> Result<String> + Send + Sync`. The command adapter calls the existing Whisper engine and joins segment text; it does not call transcript_generate or save a Transcript.
- `DictationState::start(input: DictationInput, permission: Arc<dyn PermissionBackend>, capture: Arc<dyn CaptureBackend>, pause_player: Arc<dyn Fn() -> Result<()> + Send + Sync>, still_allowed: Arc<dyn Fn() -> bool + Send + Sync>, lease: JobLease, emit: Arc<dyn Fn(DictationEvent) + Send + Sync>) -> Result<()>` uses injectable adapters. `still_allowed` rechecks current file/window focus immediately before capture; `pause_player` delegates to existing PlayerManager only after permission/model readiness. JobLease is Whisper resource from plan 2, held until the engine has actually stopped, not merely until cancellation is requested.

- [ ] **Step 1: Add failing tests** `cancel_while_permission_pending_never_starts_capture`, `permission_denied_preserves_playback`, `missing_model_never_downloads`, `recording_pauses_once_without_resume`, `stop_timeout_cancel_has_one_terminal_event`, `engine_busy_rejects_without_device_open`, `close_during_whisper_discards_result_and_releases_lease`, `dictation_does_not_write_transcript_cache`. Use fake permission callbacks and a blocking fake engine, not a real mic/key/model.
- [ ] **Step 2: Run red:** `cargo test --test dictation_pipeline` plus `cargo test jobs::`; verify expected new lifecycle failures.
- [ ] **Step 3: Implement the state machine.** Validate canonical path/model before permission. Hold a cancellable pending session, recheck it after permission resolves, acquire/retain the Whisper lease before capture, pause media immediately before opening the stream and never resume automatically. A timer stops at 60 seconds; stop closes capture before resampling/Whisper. Cancellation closes the stream promptly and signals Whisper's abort flag; do not release engine exclusion until the worker exits. Window-close cleanup never waits on the UI thread for model completion. Typed errors contain no raw audio/source text or private paths.
- [ ] **Step 4: Run green:** those tests plus `cargo test --test summary_pipeline`; verify cancellation during permission does not enter recording even if OS grant arrives later, and HTTP mock request count remains zero through dictation-ready.
- [ ] **Step 5: Commit** with `feat: add permission-safe offline dictation jobs`.

### Task 3: Mic control and non-destructive draft review

**Files:** dictation TS types/service/store, DictationControl, ChatComposer slot wiring/App listeners, locales; dictationStore/dictationControl/inlineChat tests.

**Interfaces:**

- `useDictationStore()`: `phase: 'idle' | 'requesting_permission' | 'recording' | 'transcribing' | 'review'`, `proposal: string | null`, `elapsedMs`, `errorCode`; methods `start(): Promise<void>`, `stop(): Promise<void>`, `cancel(): Promise<void>`, `insert(): boolean`, `discard(): void`, lifecycle listener registration/cleanup.
- `insert()` verifies path/session identity, then calls plan 2's `chat.appendDraft(proposal)` against the draft at insertion time; use one newline separator only when both strings are nonempty. Return true and clear the proposal only on success. If insertion exceeds 8192 bytes, keep draft/proposal unchanged and show a localized length error for review/editing. Do not store an old draft and restore it over newer edits.
- `DictationControl` renders mic/Stop/Cancel, elapsed time, paused-playback note, transcribing status and review proposal with Insert/Discard. It owns no HTTP and never emits composer Send. Focus returns to composer after review/cancel.
- `dictationService` wraps only the three IPC commands. App registers events once; file/view/visibility/window-focus watchers cancel capture, except blur caused by the pending OS permission dialog. When focus returns, verify session before proceeding.

- [ ] **Step 1: Add failing mounted/store tests** `mic_click_requests_once`, `second_click_stops_not_sends`, `hide_or_switch_file_cancels_mic`, `permission_dialog_blur_is_not_hidden_recording`, `late_result_ignored`, `edited_draft_append_is_reviewed`, `discard_keeps_current_draft`, `whisper_missing_leaves_text_chat_enabled`, `ime_enter_during_recording_never_sends`. Assert:

```ts
chat.setDraft("แก้ไขใหม่");
dictation.proposal = "ข้อความจากไมค์";
dictation.insert();
expect(chat.draft).toBe("แก้ไขใหม่\nข้อความจากไมค์");
expect(chatService.send).not.toHaveBeenCalled();
```

Capture and transcription phases disable Send; recording cancellation and review do not clear typed content. A no-media state never offers working capture.

- [ ] **Step 2: Run red:** `npx vitest run tests/unit/dictationStore.test.ts tests/unit/dictationControl.test.ts tests/unit/inlineChat.test.ts`.
- [ ] **Step 3: Implement the interfaces and approved mic UI.** Show real recording state, not decorative animated listening when idle. Provide permission/settings/model/busy error paths without opening OS settings or downloading automatically. Cancel on actual app blur after permission resolution, concealment or file change. A still-pending permission callback must not start capture in an unfocused app. Keep review readable at 640x400, with localized accessible labels/status (not a 10Hz live-region announcement).
- [ ] **Step 4: Run green:** those tests, `npm run typecheck` and `npm run lint`. Test both locales, keyboard-only insertion/cancel and concurrent draft edits through real component handlers.
- [ ] **Step 5: Commit** with `feat: add reviewed microphone drafts to inline chat`.

### Task 4: Packaged permissions, visual QA and full plan-set verification

**Files:** Info.plist/localized strings/bundle config, `tests/release/microphone-manifest.check.mjs`, relevant tests, Thai CHANGELOG and `design-qa.md`.

**Interfaces:**

- Info.plist key `NSMicrophoneUsageDescription`; English **Velo uses the microphone to turn your spoken question into editable text on this device. Audio is not uploaded.**; Thai **Velo ใช้ไมโครโฟนเพื่อแปลงคำถามที่พูดเป็นข้อความให้ตรวจแก้ในเครื่อง โดยไม่อัปโหลดเสียง**.
- Tauri bundle `macOS.infoPlist` points to the saved plist. Bundle `macOS.files` maps localized InfoPlist.strings into Resources/en.lproj and Resources/th.lproj. Inspect `node_modules/@tauri-apps/cli/config.schema.json` to validate mapping and verify actual bundled locations; do not weaken capabilities/CSP or add camera permission.

- [ ] **Step 1: Write failing manifest tests** that parse the saved plist/strings, require both translations and the configured bundle mapping, assert no camera/system-audio permission, unchanged minimumSystemVersion/version, and no secret/audio resource in the bundle. Add final integration regressions for startup without consent, text-only fallback, and recording hidden by compact player.
- [ ] **Step 2: Run red:** `node --test tests/release/microphone-manifest.check.mjs` and targeted dictation tests. Confirm intended missing declaration/integration failures.
- [ ] **Step 3: Add declarations/resources and repair those regressions.** Document first-run consent and local speech-model requirement without claiming model accuracy. Never reset the user's OS privacy database, use their key, replace `/Applications/Velo.app`, or capture a physical mic without a deliberate user-assisted recording.
- [ ] **Step 4: Run final automated gate**, saving command outcomes separately from skips:

```bash
cd src-tauri
cargo fmt
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

```bash
npm run typecheck
npm run lint
npx vitest run
npm run version:check
node --test tests/release/*.check.mjs
npm run tauri build -- --bundles app --target aarch64-apple-darwin
git diff --check
```

Run Rust tests with an explicit synthetic `VELO_TEST_MEDIA` and existing model only if available. Live Whisper/Ollama tests that skip must be listed, not represented as live passes. Windows compilation is required for the new HWND/capture paths; real-window/device QA remains separate. Compare reference/mock and native capture side by side at matching sizes; inspect welcome, full transcript, inline chat, player shelf, 640x400/1440x1024, dark/light, en/th, keyboard/IME, resize/Retina/fullscreen. Use a separate QA identifier/data directory with auto-update disabled only there. Physical mic permission/Thai recognition are user-assisted or explicitly unverified; synthetic samples test plumbing, not real recognition quality.

- [ ] **Step 5: Commit** with `test: verify packaged dictation and summary tools`. Inspect final diff and QA report; keep all six checks and bundle build evidence. No tag/push/release follows without a new user request.

## Handoff

Human review of all three plans and execution-method selection precedes any task. Native is recommended for this tightly connected set: one implementer retains the shared interfaces and does not pay repeated setup costs; arrange an independent whole-branch review only after the human selects that method. Subagent-driven is also available if the human prefers per-task independent reviews. Do not dispatch agents while awaiting that choice.
