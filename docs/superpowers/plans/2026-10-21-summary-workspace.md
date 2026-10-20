# Summary-First Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make summary the default reader, expose transcript/chat/player through three icons, and fit real native video into an optional shelf.

**Architecture:** A dedicated Pinia workspace store owns view state, independent of media/transcript data. Existing readers stay mounted; only visibility and layout change. A validated, revisioned viewport command fits the existing native mpv surface without copying video frames.

**Tech Stack:** Vue 3, TypeScript, Pinia, existing Lucide/theme primitives, Tauri 2, Rust, AppKit/OpenGL and Win32 child HWND.

**Spec:** [Approved design](../specs/2026-10-21-reading-tools-chat-design.md).

## Global Constraints

- This is plan 1 of 3; continue with [content chat](2026-10-21-content-chat.md), then [dictation](2026-10-21-composer-dictation.md).
- Status: ready for human plan review; no implementation or test completion claimed.
- Preserve all existing dirty work on `codex/summary-prompt-controls`; review its diff before editing and commit only task-owned changes.
- Summary default; no text tabs; rail order transcript, chat, player. Chat/player initially closed, no automatic inference/download/permission request.
- Main title **สรุปเนื้อหา / Content summary**; transcript **บทถอดเสียง / Transcript**; chat **คุยต่อจากสรุป / Discuss this summary**.
- Use approved loaded/welcome PNGs under `docs/superpowers/specs/assets/`; do not implement raster screenshots as UI.
- Keep both readers mounted, retain search/scroll/drafts, use one mpv instance and no video-frame IPC or HTML video.
- Minimum window 640x400; reader at least 160 logical pixels high; themes/locales both supported.
- `PLAN.md` targets macOS 12+, while the current bundle declares 11.0. Do not raise either floor in this work; build using the unchanged bundle target and report discrepancies.
- UI strings in `src/locales/en.ts` and typed `th.ts`; comments/commits English; changelog Thai. No release/version bump/push/installed-app replacement.

## Review Focus

1. Fresh installation with no model, key or media: welcome works without permission, AI probe or download (Task 1).
2. A high-DPI portrait video after zoom/resize: whole image fits, coordinates never escape the client area (Task 3).
3. Delayed viewport work after closing/reopening the shelf: old geometry cannot restore the wrong rectangle (Task 2).
4. Space on a focused rail/transport button: one action, no duplicate global shortcut/toast (Tasks 1 and 4).
5. Short/narrow windows with active search and a draft: tools remain reachable and data survives (Tasks 1 and 4).

---

## File Structure

- Create `src/stores/workspaceStore.ts`: summary/transcript view, chat preference, shelf/video-mode visibility.
- Create `src/components/workspace/WorkspaceRail.vue`, `WorkspaceWelcome.vue`: navigation and no-media entry.
- Create `src/components/player/PlayerShelf.vue`: real preview rectangle and existing transport controls.
- Create `src/types/workspace.ts`, `src/composables/useVideoViewport.ts`: typed geometry and layout observer lifecycle.
- Create `src-tauri/src/player/viewport.rs`: pure validation/conversion and revision state.
- Modify `src/components/transcript/TranscriptPanel.vue`, `SummaryTab.vue`, `TranscriptTab.vue`, `src/stores/transcriptStore.ts`, `src/App.vue`: compose the new layout; keep compatibility paths for large-video mode.
- Modify `src/services/playerService.ts`, `src-tauri/src/commands/player.rs`, `commands/mod.rs`, `player/mod.rs`, `platform/macos.rs`, `platform/windows.rs`, `lib.rs`: viewport IPC and native surface integration.
- Modify `src/components/player/TopBar.vue`, `ReadingTransport.vue`, locales and styles only where this layout needs them.
- Create `tests/unit/workspaceTools.test.ts`, `videoViewport.test.ts`, `tests/unit/helpers/mountView.ts` and `src-tauri/tests/video_viewport.rs`; retain/update existing reading/keyboard tests.

### Task 1: Summary-first shell and independent tool state

**Files:** workspace store/types/components, TranscriptPanel, transcriptStore, SummaryTab, locales, `tests/unit/workspaceTools.test.ts`, `helpers/mountView.ts`.

**Interfaces:**

- `useWorkspaceStore()`: `view: 'summary' | 'transcript'`, `chatOpen: boolean`, `playerOpen: boolean`, `mode: 'reading' | 'video'`; methods `toggleTranscript(): void`, `toggleChat(): void`, `togglePlayer(): void`, `resetForMedia(): void`, `showVideo(): void`, `showReading(): void`.
- `chatVisible` is computed from `view === 'summary' && chatOpen && mode === 'reading'`; transcript switching retains `chatOpen` preference.
- `toggleChat()` from transcript selects summary and sets chatOpen true; from summary toggles it. `resetForMedia()` selects summary and closes both tools.
- Existing `transcriptStore.showVideo/showReading` delegate to workspace; migrate UI consumers of its reading-mode fields rather than retaining two independent sources of truth.
- `WorkspaceRail` consumes these store actions; emits no generation or playback commands. It exposes stable `data-testid` values `tool-transcript`, `tool-chat`, `tool-player`.
- `mountView(component, props, pinia)` returns `{ find(testId): TestNode, unmount(): void }`; `TestNode.props` records attributes and `trigger(event, payload?)` invokes the real Vue handler. Extend the existing custom renderer approach; no new DOM dependency.

- [ ] **Step 1: Add failing behavior tests**, including the following assertions and real mounted rail-button clicks/keydown handlers:

```ts
const workspace = useWorkspaceStore();
expect([workspace.view, workspace.chatOpen, workspace.playerOpen]).toEqual([
  "summary",
  false,
  false,
]);
workspace.toggleTranscript();
expect(workspace.view).toBe("transcript");
workspace.toggleChat();
expect([workspace.view, workspace.chatVisible]).toEqual(["summary", true]);
workspace.toggleTranscript();
expect([workspace.chatOpen, workspace.chatVisible]).toEqual([true, false]);
workspace.toggleTranscript();
expect(workspace.chatVisible).toBe(true);
expect(invoke).not.toHaveBeenCalled();
```

Add `fresh_welcome_never_requests_ai_model_or_mic`, `new_file_resets_only_visibility`, `rail_space_activates_once`, `chat_from_transcript_returns_to_summary`, `cached_summary_reads_without_provider`, and `reader_search_scroll_survives_tool_changes`. App-start tests may allow existing local status/settings/cache calls and the configured updater; assert that inference/probe/download/mic calls are absent rather than prohibiting all IPC. Verify no old tablist remains and all three buttons have labels/state in both locales. No media has precedence over model/provider setup; a cached summary remains readable before setup gates.

- [ ] **Step 2: Run red:** `npx vitest run tests/unit/workspaceTools.test.ts tests/unit/readingWorkspace.test.ts`. Expect new API/behavior assertions to fail; record the specific failure, not unrelated build errors.
- [ ] **Step 3: Implement the store/actions and shell.** Keep reader instances mounted with `v-show`, use existing open-file/folder/drop actions, keep no-file icons inactive with the spec's explicit empty states. Rename presentation copy only, not provider summary output headings. Until plan 2, chat opens an honestly labeled unavailable state through a named slot; no canned AI response or nonfunctional composer.
- [ ] **Step 4: Run green:** the same Vitest command plus `npm run typecheck` and `npm run lint`; expect zero failures/errors. Update old tests that explicitly required transcript-first startup without weakening unrelated assertions.
- [ ] **Step 5: Commit task-owned paths only** with `feat: add summary-first workspace navigation`; inspect staged diff before committing.

### Task 2: Validated viewport contract and observer lifecycle

**Files:** `src/types/workspace.ts`, `useVideoViewport.ts`, playerService; Rust viewport module, commands/reexports/lib registration; `videoViewport.test.ts`, `video_viewport.rs`.

**Interfaces:**

- `VideoViewportInput { rect: { x:number; y:number; width:number; height:number }; css_width:number; css_height:number; visible:boolean; revision:number }` serialized with matching Rust field names; revisions are monotonic integers per window, not reset per component mount.
- `playerService.setVideoViewport(input: VideoViewportInput): Promise<void>` invokes `player_set_video_viewport(input)`.
- Rust `ViewportState::apply(input: VideoViewportInput, client: ClientMetrics) -> Result<Option<ResolvedViewport>>`; `ClientMetrics { width: f64, height: f64, scale_factor: f64 }` in logical units. Old/equal revisions return None; invalid values error without advancing revision. `ResolvedViewport` contains the clamped logical rect and visible flag.
- `useVideoViewport(element: Ref<HTMLElement | null>, visible: () => boolean): void` observes layout, window/scale/scroll changes, coalesces via requestAnimationFrame, and removes listeners/observers on scope disposal.

- [ ] **Step 1: Write failing pure Rust/observer tests.** `rejects_nonfinite_or_nonpositive_metrics`, `clamps_negative_edges_and_oversize_rect`, `converts_css_zoom_to_client_logical_units`, `ignores_old_revision_after_close`, `observer_disposal_cannot_reopen_surface`. Pin an input rect `{x:100,y:50,width:400,height:200}`, CSS 1000x500 and logical client 1200x600 to output `{x:120,y:60,width:480,height:240}`. Assert NaN, infinity, invalid revisions and stale updates never reach the platform adapter; visibility-only updates do not call seek/play/load.
- [ ] **Step 2: Run red:** `cargo test --test video_viewport` in `src-tauri`; `npx vitest run tests/unit/videoViewport.test.ts` in repo root. Expect missing contract or failing assertions.
- [ ] **Step 3: Implement the contract and observer.** Resolve from CSS viewport dimensions to webview logical client coordinates, then clamp. Return an acknowledged revision; surface IPC failures show an actionable localized shelf error. Keep revision tracking in a persistent window-scoped service, not a component-local counter. Native application always runs on the UI thread, with a revision check there as well.
- [ ] **Step 4: Run green:** both commands above plus typecheck; verify multiple observer notifications produce one update per animation frame and teardown cannot send an older visibility update afterward.
- [ ] **Step 5: Commit** with `feat: add validated native video viewport contract` after staged diff review.

### Task 3: Aspect-fit native shelf on macOS and Windows

**Files:** `PlayerShelf.vue`, `ReadingTransport.vue`, TranscriptPanel, App/styles, macOS/Windows platform modules, lib resize handling, `src-tauri/Cargo.toml` Windows feature flags if required, native viewport tests.

**Interfaces:**

- `PlayerShelf` consumes workspace/player stores, uses `useVideoViewport` on its transparent preview only, and reuses existing transport actions. Stable IDs `player-shelf`, `video-preview`, `close-player`, `expand-player`.
- Platform `apply_video_viewport(window_handle, resolved: &ResolvedViewport) -> Result<()>` executes on the window UI thread; stored viewport is reapplied by resize/scale handlers.
- macOS retains the existing GL view/render context. Windows `setup_video_surface(parent_hwnd)` returns an owned child HWND used as mpv's wid; lifecycle is managed alongside the main window, not recreated on every toggle.

- [ ] **Step 1: Add failing tests** `appkit_rect_flips_y_once`, `win32_rect_scales_and_rounds_edges`, `hidden_surface_retains_nonzero_live_render_target`, `shelf_close_does_not_pause_audio`, `audio_only_has_no_empty_video_hole`, `modal_masks_native_preview`. For a logical window 800x600 and rect x100/y50/w300/h200, AppKit y must be 350. At scale 2 the Win32 rectangle must be x200/y100/w600/h400. Test out-of-order UI-thread updates too.
- [ ] **Step 2: Run red:** `cargo test --test video_viewport` and `npx vitest run tests/unit/workspaceTools.test.ts tests/unit/videoViewport.test.ts`; expect newly required platform/shelf behavior to fail.
- [ ] **Step 3: Implement adapters and shelf composition.** Replace macOS full-bounds reset with stored viewport application and backing conversion; preserve mpv draw/swap callbacks. Create the Windows child with pointer-transparent hit testing and keep it below the webview; enable `Win32_System_LibraryLoader` and `Win32_Graphics_Gdi` for child-class/module-handle setup without changing windows-sys version. Hidden means a live native target covered by an opaque webview, not zero size or destroying its GL context. Do not stop rendering in a way that stalls audio. Only the exact preview opens a transparency hole; masks cover it for dialogs/overlays. Retain native aspect fitting; no screenshot/canvas thumbnail substitute. Large-video expansion reuses the existing mode and returns to the open shelf.
- [ ] **Step 4: Run green** for those tests and `npm run build`. Build the actual bundle: `npm run tauri build -- --bundles app --target aarch64-apple-darwin`. Verify moving landscape/portrait synthetic media visually; do not claim native success from coordinate unit tests alone.
- [ ] **Step 5: Commit** with `feat: fit native playback into an optional media shelf`; inspect for unchanged deployment target and preserved player ownership.

### Task 4: Responsive/accessibility and real-app regression gate

**Files:** affected workspace components/styles/locales, reading/keyboard tests, `design-qa.md`; no unrelated refactors.

**Interfaces:**

- A `matchMedia`/ResizeObserver-derived compact layout ensures a 160px reader minimum. If the open shelf violates it, opening chat closes the shelf and opening the shelf conceals chat without clearing its preference.
- Escape closes the focused tool and restores its trigger; modal/input ownership wins. Reading scroll is not reset by toolbar layout changes.

- [ ] **Step 1: Write red tests** `compact_tools_preserve_draft_and_search`, `escape_restores_own_trigger`, `focused_button_space_never_reaches_global_player`, `welcome_open_file_uses_existing_dialog`. Test dimensions 640x400 and 1440x1024, both locales; assert visible state through mounted handlers, not string snapshots alone.
- [ ] **Step 2: Run red:** `npx vitest run tests/unit/workspaceTools.test.ts tests/unit/keyboardShortcuts.test.ts tests/unit/readingWorkspace.test.ts`. Confirm expected new failures.
- [ ] **Step 3: Implement only failing responsive/focus/copy details.** Make starter/toolbar rows wrap and prevent rail overlap with the playlist or macOS traffic lights. Do not add separate live chat code here.
- [ ] **Step 4: Run the six repository checks** and actual isolated macOS QA. Compare selected PNG and app captures at matched viewport/state in one comparison; inspect welcome, summary, transcript, shelf, both themes, keyboard editing, resize/Retina/fullscreen, audio playback with shelf hidden. Record Windows compilation/real-window evidence separately or explicitly leave it unverified. Record optional model/live skips; do not capture a real microphone.
- [ ] **Step 5: Commit** with `test: verify summary workspace and native shelf`; include the QA report, preserving prior report entries. The part-1 deliverable is a working summary/transcript/player workspace, not a completed AI chat feature.

## Handoff

Review all three plans and select Native or Subagent-driven before product changes. Preserve the chosen method across the plan set. This plan adds no cloud call, mic recording, PR, tag or release authorization.
