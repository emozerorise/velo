# Velo transcript-first workspace QA

## 2026-10-22 — Sidebar branding (selected option 2)

Result: **passed** for the changed macOS sidebar header and import controls.

- Selected reference: `exec-b023dbcb-a0a5-44f8-990c-0297a764943c.png`. Reviewed alongside `/private/tmp/velo-brand-wide.png` in one comparison input. The reference is an enlarged 1840×854 header crop; the implementation uses the existing 280 CSS px sidebar, captured at 2× density. Comparison is proportional rather than pixel-identical.
- Blue rounded tile reuses the existing Lucide waveform; the compact Velo wordmark uses already bundled Prompt independently of the reading-font preference. Navy primary import button and neutral folder button follow the selected hierarchy. No application bundle icon was changed.
- Intentional adaptations: 44 px button targets, existing native macOS window controls, narrow-sidebar close button, and flat themed surfaces instead of the reference's raster shading.
- Native evidence: `/private/tmp/velo-brand-narrow.png` (640×600 window), `/private/tmp/velo-brand-wide.png` (1728×1027 window), `/private/tmp/velo-brand-dark.png`. No clipping or overlapping header controls observed in either viewport. Light and dark theme text/icons remain legible.
- Keyboard: Tab reaches file then folder controls; Return opens the corresponding native chooser. Escape cancels each chooser and restores focus. No files were imported. Original Preview light theme restored after dark-theme inspection.
- Checks passed: typecheck, lint, 292 Vitest tests across 32 files, cargo fmt, clippy with warnings denied, cargo test, git diff --check, and the aarch64 native Preview app bundle build. Opt-in live AI/audio tests were not enabled; this visual-only task made no API calls.
- No unresolved findings in the changed scope. Isolated Velo Library Preview left open for user review. Production data unchanged; no push or release.

## 2026-10-22 — Typography settings (selected option 2)

Result: **passed** for the macOS desktop scope.

- Reference: `exec-479ed077-1293-45fc-908e-9bd09d9c6977.png` in the conversation's generated images. Compared alongside `/private/tmp/velo-font-settings-final.png` in the same review input.
- Implemented the two grouped font columns and shared reading preview in the existing Settings modal, not a separate prototype. Real bundled glyphs replace the mockup's illustrative typography. Existing design tokens, modal header, close control and tabs are retained.
- Six new families: Sarabun, Noto Sans Thai Looped, IBM Plex Sans Thai Looped, Prompt, Kanit, IBM Plex Sans Thai. Original Noto Sans Thai remains the default/reset option. Fonts and per-family OFL licenses are local (about 1.6 MB added); no runtime font service.
- Layout/typography: reviewed the 640×600 native window with scrolling and the zoomed 1728×1027 window. Names, Thai vowels, sample text and buttons remain readable without horizontal clipping. The shared preview keeps 16 px / 1.8 reading rhythm; heading is semibold. Mono sample remains unchanged. The desktop app's minimum width is 640, so mobile is not a supported target.
- Colors/states: reviewed light and dark themes, checked radio highlight, keyboard focus rings, hover, disabled Apply, and saved confirmation. No imagery or new decorative icons are required by this design.
- Native interactions: arrow keys selected all six fonts; Tab reached reset/apply; Enter applied the selected font. Draft selection does not apply globally. Escape and reopen discarded the Prompt draft. Reset previews the original font and requires Apply.
- Persistence: applied IBM Plex Sans Thai, confirmed `font_family` on disk in the isolated Preview data directory, quit/relaunched, and verified its selected radio and preview. Subsequent theme save preserved the font. Finally restored the Preview's original Noto Sans Thai and light theme. Production application data was not modified.
- Automated checks: 291 frontend tests across 32 files passed; typecheck and lint passed without warnings. `cargo fmt`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed. New tests cover all six previews, draft discard, explicit Apply/reset, failed writes, load restoration, legacy/unknown preferences, and disk round trips preserving unrelated settings.
- Native app bundle build passed for aarch64-apple-darwin with the isolated Library Preview config. Existing opt-in live AI/audio tests retain their skip/ignore behavior; no API calls were made for this font task.
- No push, tag, release or production installation performed.

## Target and evidence

- Selected visual: `/Users/emozerorise/.codex/generated_images/01a0593d-0d1f-72d3-aaa9-4a545bcdbf53/exec-ff6d3640-7e08-4bb9-8e4b-f796d4896416.png` (second displayed option).
- Reference pixels: 1487 × 1058. Native QA viewport: 1440 × 1024 CSS px, 2× density; captures 2880 × 2048.
- First implementation: `/private/tmp/velo-reading-first.jpg`, normalized to reference size in `/private/tmp/velo-reading-first-normalized.jpg`.
- Reference and normalized implementation were opened together in one comparison input. Native macOS window controls/sharing indicator, synthetic filename, and fewer playlist entries are intentional differences. The first capture follows the playhead, so its first timestamp is above the scroll edge; final comparison must use scroll-to-top.
- Video-state evidence: `/private/tmp/velo-reading-video-first.jpg`, showing the synthetic test pattern at 00:37 and the transcript drawer.

## Findings — iteration 1

- [P2] Reading typography and sidebar proportions are smaller than the selected mock. Increase the sidebar from 240 to 272 CSS px, heading and body size, and match column indentation without increasing paragraph height.
- [P2] Native volume slider has an oversized platform thumb unlike Velo's existing range styling. Reuse the themed range track and thumb.
- [P2] Video toolbar remains underneath the transcript drawer. Constrain the top toolbar as well as the transport to the visible video area, and verify narrow-window controls.
- [P3] Keep Velo's existing brand text and library audio icon instead of inventing a separate logo asset. No photography/raster assets are required by the selected visual.

## Required surfaces

- Typography: system sans-serif/Thai fallback as the existing product; body size and heading hierarchy need the iteration above.
- Spacing/layout: centered reading column and persistent footer work; sidebar proportion and video toolbar need fixes.
- Colors/tokens: existing opaque chrome/surface and blue accent retained; native video is completely covered only in reading mode.
- Assets/icons: existing Lucide outline family; no decorative image placeholders or replacement raster assets.
- Copy/content: bilingual UI; synthetic transcript only, no real meetings or model calls. Technical language code can remain as metadata.

## Interaction evidence so far

- Starts in reading mode without a file.
- Opens media from the sidebar; cached transcript reads even without a speech model installed.
- Clicking 00:37 seeks to 37 seconds.
- Watch video shows native video with the transcript on the right; return keeps playback paused at 37 seconds.
- Search for `สมมติ` returns two rows and persists across reading/video switches.
- Summary tab and manually saved cached summary remain available without regeneration.

## Remaining checks

- Re-capture after fixes; compare full view and header/footer detail.
- Small window, light theme, playback controls, speed, playlist navigation, summary prompt/time controls, and all command-line checks.

## Iteration 2 — minimum window

- `/private/tmp/velo-reading-small-first.jpg` is 1280 × 800 pixels (640 × 400 CSS px at 2×). [P2] The tall reading heading left only one line visible. Use a shorter header/title rhythm below 550px height, omit redundant metadata there, and keep the footer controls compact.
- `/private/tmp/velo-reading-small-video.jpg` confirms compact playback, time, speed and fullscreen fit beside a 320px drawer. [P2] TopBar's own `w-full` overrides the right inset; explicitly set its constrained width to auto.
- Manual window-edge resize was not reliable through the native automation surface. These captures use an isolated QA build with the actual minimum window configuration, not browser zoom.
- Extra headless video tests `idle_active_reports_whether_a_file_is_loaded` and `loads_a_file_at_a_resume_position` timed out waiting for `FILE_LOADED` with the video-only fixture. Their core uses `vo=libmpv` without the native render context the full app creates. The same two tests pass with `/private/tmp/velo-reading-audio.wav` (2/2), and the actual app displays and seeks the video fixture correctly. This headless-video harness limitation is not represented as a passing video test.

### Post-fix evidence

- `/private/tmp/velo-reading-small-final.jpg`: 640 × 400 reading now exposes a complete current paragraph and persistent transport. `/private/tmp/velo-reading-small-video-final.jpg`: video toolbar, pause/play, time, speed and fullscreen are visible beside the 320px drawer. `/private/tmp/velo-reading-small-light.jpg`: light tokens preserve readable type, timestamp links and active-paragraph highlight. Both minimum-window P2s are resolved.
- Minimum-window sidebar opens and dismisses; settings is reachable. Video playlist opens without hiding behind the transcript; Transcript returns to the full reader. Playback remains paused at the same time.
- Root cause of the extra video-test timeout was confirmed by changing **only the test harness** to `vo=null`, appropriate for state/resume tests with no render surface. Both named tests now pass with the original video fixture (2/2, 0.47s). Production video rendering is unchanged and separately verified in the native app.

## Iteration 3 — full view and scroll-state verification

- Final design comparison used the selected mock and `/private/tmp/velo-reading-final-normalized.jpg` together in one comparison input. This normalizes `/private/tmp/velo-reading-final.jpg` (2880 × 2048) to 1487 × 1058. Both show paused 00:37, dark theme, transcript tab, and the transcript scrolled to its start. The 272px sidebar, 32px heading, 18px/1.7 body, restrained active row and themed footer resolve iteration 1's type/proportion/slider findings.
- Copy/regenerate remain as existing compact actions below the tabs rather than being hidden in the mock's overflow menu. This is an intentional discoverability difference, not missing functionality. Native window chrome, the synthetic filename/content, one real playlist item rather than three invented ones, and the technical language code are also intentional.
- Header/footer controls and Thai text are clearly readable in the full-resolution captures; separate crops are not needed for these asset-free UI surfaces. Icon strokes, type hierarchy, wrapping, contrast, active state, dividers, and transport were inspected at full resolution.
- `/private/tmp/velo-reading-scroll-before.jpg` versus `/private/tmp/velo-reading-scroll-after.jpg` exposed a remaining [P2]: parent resize clamps the scroll position before a child's pre-flush watcher samples it. The earlier unit test modeled the clamp too late. The regression now clamps before that watcher, fails against the prior implementation, and the fix remembers real scroll events instead. A new native capture is required before clearing this finding.
- The all-suite run with a silent audio fixture correctly failed `extracts_16khz_mono_wav` because that test requires a non-silent signal. Re-running the complete Rust suite with `/private/tmp/velo-reading-tone.wav` (synthetic 440Hz tone) passes, including extraction. No production extraction change was made.

## Post-fix acceptance

- `/private/tmp/velo-reading-scroll-fixed-before.jpg` and `/private/tmp/velo-reading-scroll-fixed-after.jpg` were opened together. At paused 03:00, the same 01:03 paragraph starts the viewport and the current 03:00 block stays fully visible at the bottom after a video/read round trip. The scroll-event implementation resolves iteration 3's last P2; the old before/after mismatch is retained above as regression evidence.
- The final full-reader capture is `/private/tmp/velo-reading-final.jpg`. The source mock and normalized final full view were compared together; the subsequent scroll fix changes behavior only, not the visual system. All earlier P2 findings have post-fix native evidence, with no remaining P0/P1/P2 issues.
- Fonts/type: system sans/Thai fallback, 32px heading, 18px/1.7 text and 14px time links are readable; the compact view deliberately uses smaller type. Spacing/layout: 272px sidebar, centered reading column, persistent transport, short-window heading, and constrained video toolbar checked. Colors: dark/light tokens, active row, readable button labels and time links checked. Assets: library outline icons and real native video; no fabricated image assets. Copy: English/Thai labels, setup/privacy copy, saved prompt and timestamp controls retained.
- Native interactions checked: default empty/read startup; file opening; cached transcript without a model; seek from rows, keyboard and ±5 buttons; playback/pause; 1.25x speed; mute; search and query retention through view switches; summary tab; time hiding; existing prompt editor without generation; sidebar open/dismiss/settings; video playlist and return to full reader; reading scroll restoration.
- Verification: all six requested commands pass, frontend 106 tests/15 files; Rust 40 unit tests plus integration suite, including real synthetic-audio extraction and state/resume. Additional video-fixture state/resume tests pass 2/2 after the headless harness fix. Release-manifest tests 3/3, version consistency and diff whitespace checks pass. Apple Silicon app bundles build successfully. The existing STATIC_VCRUNTIME deprecation warning is unrelated and not a build failure.
- Residual scope: no new live Whisper inference/model download, cloud API call, or signed-update archive test; those environment-gated tests retain their opt-in behavior. Windows/Intel runtime testing is not available on this host. Native webview console access is not exposed by the UI tool, so no browser-console sweep is claimed. These are test-environment limits, not asserted passes.
- Follow-up polish (P3): metadata currently shows `th` rather than the mock's full language name; copy/regenerate stay visible instead of an overflow menu. These intentional product-preserving differences do not block the selected reading/video workflow.

final result: passed
## Summary-first workspace — 21 October 2026 (in progress)

- Source: `docs/superpowers/specs/assets/2026-10-21-summary-first-welcome.png` (1487×1058), plus the approved inline-chat and original player-shelf references in the same directory.
- Native capture: `/private/tmp/velo-workspace-welcome-pass1.png` (2880×2048), 1440×1024 CSS at 2× density. Both sides normalized to 1487×1058; combined evidence `/private/tmp/velo-workspace-welcome-comparison-pass1.png` is 2974×1058, source left / app right. Aspect ratios differ by less than 0.1%.
- State: Thai, dark, no media, all rail icons inactive. Purple sharing badge is macOS chrome.
- [P2] Typography/navigation density: welcome heading, helper copy, main title and rail icons are visibly smaller than the reference, weakening the welcome hierarchy. Increase reading/welcome type scale, rail size and welcome emblem/CTA, keeping existing theme tokens.
- [P2] Empty playlist hierarchy: source has a no-files title and explanatory line; app only shows a small drag hint. Add the explicit empty state with an existing Lucide document icon.
- Shelf evidence: `/private/tmp/velo-workspace-portrait-shelf.png`, `/private/tmp/velo-workspace-landscape-shelf.png`, `/private/tmp/velo-workspace-modal-mask.png`. Both aspect ratios fit; expand/return and prompt masking work. Hidden shelf retained playback from 02:53 to 03:23. Stale background captures were investigated: render/core loops were active; foreground activation refreshed state. No speculative rendering fix was made.
- Windows adapter and viewport code passed an x86_64-pc-windows-msvc binding-check crate; full Windows app/real-window testing remains unverified.
- Current result: blocked pending post-fix comparison and minimum-window QA. AI chat/dictation are later plans, not claimed complete here.
- Welcome comparison pass 2: `/private/tmp/velo-workspace-welcome-comparison-pass2.png` uses `/private/tmp/velo-workspace-welcome-pass2.png` at the same viewport/density/state. Enlarged type/emblem/rail and explicit empty-playlist copy resolve both prior P2 findings. Existing charcoal theme and library audio-document icon are intentional product-system choices versus the generated blue-tinted reference, not new bitmap art.
- Loaded-summary inspection found a further [P2]: the separate heading plus toolbar wasted vertical space, and inherited narrow rounded cards differed from the approved plain, wide reading layout. Consolidate the summary heading/toolbar and use a borderless 18px reading presentation; keep existing compact cards in large-video mode. Section parsing, citations and five-section content are unchanged. This needs post-fix native evidence.

### Part 1 post-fix acceptance

- `/private/tmp/velo-workspace-summary-comparison-pass3.png` compares the approved inline-chat reference (left) and `/private/tmp/velo-workspace-summary-reading-pass3.png` (right), normalized to 1487×1058 per side. The app is Thai/dark at 1440×1024 CSS/2×. One heading with its toolbar, wide plain sections, 18px body and restrained blue citations resolve the loaded-summary P2. The reference has chat manually open and illustrative three-section content; this capture has chat closed and preserves the existing five-section contract. This comparison accepts the underlying reading hierarchy, not the still-unimplemented chat composer.
- `/private/tmp/velo-workspace-summary-light-en.png` verifies English/light chrome with the same cached Thai content. Cached content is not translated by changing UI language. The toolbar remains reachable, with horizontal overflow at minimum width instead of hiding actions.
- `/private/tmp/velo-workspace-small-summary-light-en.png` and `/private/tmp/velo-workspace-small-shelf-fixed.png` show 640×400 CSS/2×. Native inspection first found [P2] speed wrapping onto another row and reducing the reader to 145 logical pixels. A dedicated full-width timeline row and a non-wrapping compact action grid resolve it: the reading viewport is approximately 167 logical pixels (image y=220..554 at 2×). Play, ±5 seconds, time, mute, speed, expand and close stay visible. An additional regression was observed RED→GREEN before the final bundle.
- No-file transcript and player Open actions both open the existing native dialog. Escape closes the focused tool and restores its rail focus; Space activates it once without toggling global playback. Both locales and 640×400/1440×1024 state retention run through mounted component handlers.
- Audio-only evidence: `/private/tmp/velo-workspace-small-audio-before-hide.png`, `/private/tmp/velo-workspace-audio-hidden-fullscreen.png`, `/private/tmp/velo-workspace-small-audio-after-hide.png`. A synthetic 440Hz WAV plays with output muted, no empty video rectangle. Its clock advances from 00:04 to 00:22 across shelf hiding, fullscreen and return to the minimum window. This verifies continued audio-stream playback state, not acoustic speaker output. The earlier 02:53→03:23 check used a video-only fixture and is not claimed as audio evidence.
- Retained native evidence covers portrait/landscape aspect-fit, large-video expand/return, prompt-modal native masking, new-file visibility reset, Retina geometry and fullscreen/return. All six repository commands pass: frontend 133/133 tests, Rust full suite with the non-silent synthetic WAV, fmt/check and warning-denying clippy. Version sources remain 0.8.0; release-manifest checks pass 3/3; the unchanged-target Apple Silicon QA bundle builds.
- Limits: Windows adapter bindings compile against the actual production files, but full Windows application compilation and real-window/device testing remain unverified. Live Whisper inference/model download and live Ollama tests retain opt-in skips; no paid/cloud inference, real microphone, real keychain/settings or installed application was used. Native webview console inspection is unavailable. Chat/dictation acceptance remains in parts 2 and 3, including no-file chat focus/announcement.
- Part-1 result: passed for summary/transcript/native shelf. No unresolved part-1 P0/P1/P2 findings; full plan-set acceptance is pending.

## File-scoped inline chat — part 2 in progress

- Native functional evidence uses the separate `com.velo.summary-controls-qa` bundle at 1440×1024 CSS/2×, a synthetic cached transcript, and a disposable provider bound only to `127.0.0.1`. The QA credential namespace is isolated from the released app's existing keychain service. No real account/key, cloud inference, Whisper download or microphone was used.
- `/private/tmp/velo-chat-no-summary-light-en.png`: a cached transcript is sufficient without a summary. Opening chat and choosing a starter left mock request counters at zero; manual Send produced visible streaming text. `/private/tmp/velo-chat-answer-light-en.png` shows the retained response and validated timestamp buttons after collapse/reopen. The first response correctly retains its “No summary supplied” metadata even after a summary is generated later.
- `/private/tmp/velo-chat-prompt-answer-dark-th.png` and `/private/tmp/velo-chat-prompt-review-dark-th.png`: the second manual question produces exactly one reviewable prompt block. Review opens the existing editable modal, with its all-files scope warning; Cancel retains the previous QA instructions. An explicit edited Save changes only the QA setting. The saved summary SHA-256 remains `46991c67d0faff26fb587133fbed5fdbac1cd2dfec563adbbde3e0915f678966`. Counters remain 3 requests: 1 deliberately generated synthetic summary and 2 manually sent conversations. Prompt saving causes no request or summary rewrite.
- Mounted/store tests cover duplicate/gapped events, authoritative final replacement, cancellation/file/provider isolation, six-pair history trimming, retained partial failures/retry without duplicate user turns, IME/Shift+Enter, draft confirmation, safe HTML/link rendering, no-file welcome focus/announcement, and prompt draft/save behavior. A further RED→GREEN regression prevents theme-only settings replacement from cancelling an active response. Current frontend total is 162 passing tests; typecheck/lint, full Rust suite and unchanged-target QA bundle build pass.
- Part-2 visual gate is not complete: compare an empty chat and the approved inline reference at matched state, verify minimum-window behavior and slower native cancellation/retry in Task 5. [P2] New chat metadata/privacy copy at `fg/45` or `fg/50` is too faint in the light-theme capture; raise meaningful secondary-copy contrast in that visual gate. Microphone controls are intentionally absent until part 3, not a fabricated working button.

### Part 2 regression gate — awaiting unlocked native session

- Task 5 raises meaningful secondary chat text to `fg/60`, privacy text to `fg/65`, adds a visible composer focus ring and uses white on blue-600 for Send. Nominal computed contrasts are 4.70:1 (light secondary), 7.09:1 (dark secondary), 5.46:1 (light inset privacy), and 5.17:1 (Send icon). The earlier P2 is **not cleared by calculation alone**; updated native light/dark captures remain required.
- Mounted regressions exercise a real `TranscriptPanel` at 640×400, IME input and a registered streaming callback while compact player mode hides chat. Reopening retains the same textarea node, draft and in-progress answer. Additional cases cover a cached transcript without summary or installed speech engine, a non-meeting question, recoverable provider failure/Retry with one user turn and concurrent edits, and completion arriving before IPC acknowledgement. Frontend: 170/170 tests; typecheck and lint clean.
- Prompt review now exposes a named modal and localized Close control, owns header-button Space/Escape and cycles/restores keyboard focus. The mounted keyboard regression passes; native focus verification remains pending. No prompt save or review action sends a chat request.
- A full Rust run exposed two timing-sensitive Stop tests: their 250ms budget included connection setup. A deliberate 300ms mock setup delay reproduced both failures; measuring from actual cancellation retains the 250ms Stop assertion and makes both pass without changing production transport. The setup watchdog is separately bounded at five seconds.
- The updated isolated Apple Silicon app bundle builds at the unchanged deployment target. Version sources agree on 0.8.0; release-manifest checks pass 3/3; diff whitespace check passes. No microphone or real provider was used.
- Final host Rust gate passes `cargo fmt`, `cargo fmt --check`, warning-denying clippy and the full suite with `/private/tmp/velo-reading-tone.wav`; this includes real synthetic-audio extraction and both corrected Stop tests. Explicit opt-in skips remain: network model download, live Ollama summary, three Whisper inference cases (no model installed), and signed release-archive verification (no archive supplied). Their early-return test rows are not live passes.
- Native continuation is blocked by the locked Mac (`cua.getApp` reported that automatic unlock could not unlock it). The user has been asked to unlock; no attempt was made to bypass the lock. Updated Retry/Stop, matched-reference comparison, minimum-window and modal focus acceptance are still pending. Part 2 remains **in progress**, and part 3 microphone acceptance has not started.

### Part 2 post-fix acceptance

- After the user unlocked the Mac, native QA resumed in the same isolated application. Retry preserves concurrent draft edits and one user turn. `/private/tmp/velo-chat-retry-final-dark-th.png` shows the completed retry and retained draft. A slower streamed request was hidden/reopened while still active, then stopped; `/private/tmp/velo-chat-native-hidden-stream.txt` records the actual native AX states, including the still-disabled Regenerate and visible Stop on reopening. The cancelled partial is labeled and a later manual question succeeds.
- A real native citation bug was discovered: `[00:37]` added 37 seconds to the current position rather than seeking absolutely. The mounted regression first failed with the relative IPC call; `ChatMessage` now uses the existing absolute seek. `/private/tmp/velo-chat-citation-absolute-final.png` confirms the rebuilt app opens its previously hidden shelf paused at **00:37 / 03:44**. Source text/caches are unchanged.
- Native prompt review opens an editable draft. Shift+Tab wraps to Save, Tab wraps to the named Close button, and Space closes without starting paused playback. `/private/tmp/velo-chat-prompt-focus-final-dark-th.png` records keyboard focus. Cancel preserves the prior QA instructions; an edited explicit Save changes only QA instructions. Counters stay at five requests (four conversations, one deliberate failure, zero summaries), and summary SHA-256 remains `46991c67d0faff26fb587133fbed5fdbac1cd2dfec563adbbde3e0915f678966` across both actions. Mounted coverage additionally checks focus restoration.
- Reference `docs/superpowers/specs/assets/2026-10-21-summary-first-inline-chat.png` is compared side by side with `/private/tmp/velo-chat-reference-final-dark-th.png` in `/private/tmp/velo-chat-comparison-final.png`: source left/app right, 2974×1058. Native 2880×2048 is normalized to 1487×1058 (1440×1024 CSS, 2×). Both are Thai/dark/loaded summary/chat manually open/unsent draft. Production retains five sections rather than the reference's three illustrative sections, so the shared reader is scrolled further to expose the composer; this is not a pixel-perfect content match. The spec explicitly retains existing charcoal tokens/Lucide, compact welcome hierarchy, and the five-section contract. Mic is deliberately deferred until part 3.
- Fidelity surfaces: readable system/Thai fallback, compact heading and multiline composer; wide reading column, divider and lightweight wrapping starters; existing theme tokens, higher secondary-text contrast and visible input ring; crisp existing library icons (no missing/generated raster assets); neutral bilingual labels and explicit destination/session copy. Full-resolution chat/privacy/focus controls were inspected directly; no focused crop is needed to read these asset-free controls. The earlier light-theme contrast P2 is resolved by post-fix native evidence, not only contrast calculations.
- `/private/tmp/velo-chat-small-final-light-en.png` shows the 640×400 CSS/2× English/light composer with readable privacy/session copy, Thai Unicode draft, visible Send and focus ring. Shift+Enter leaves it unsent. Mounted tests cover actual composition events; OS IME candidate interaction is not claimed. `/private/tmp/velo-chat-small-player-final.png` shows compact player mode concealing chat without collapsing the reader below 160 logical pixels. Reopening while streaming retains the new draft and Stop; Stop ends the request without losing edits. Transcript search `สมมติ`, conversation and draft survive full-reader round trips. The shared timestamp preference removes all summary/chat seek buttons without inference.
- Fresh automated gate passes all six commands, `cargo fmt --check`, version consistency (still 0.8.0), release checks 3/3 and whitespace. Frontend **171/171**; Rust suite passes with the synthetic non-silent WAV. The same explicit live/model/network/archive skips listed above remain skips. Updated large and minimum-window Apple Silicon bundles build at the unchanged target.
- Part-2 result: no unresolved P0/P1/P2 findings. Native console introspection, live model quality, Windows runtime and real microphone remain unverified. Part 3 and final whole-branch review are still pending; no push/release/install replacement performed.

## Part 3 — reviewed microphone drafts (2026-10-21)

- No actionable P0/P1/P2 visual findings in this pass. Source remains `docs/superpowers/specs/assets/2026-10-21-summary-first-inline-chat.png` (1487×1058). Native Thai/dark/loaded summary/open discussion/unsent draft is `/private/tmp/velo-mic-cancel-mask-dark-th.png` (2880×2048, 1440×1024 CSS, 2×). `/private/tmp/velo-mic-comparison.png` puts source left and normalized native right in one 2974×1058 comparison input. As recorded for part 2, the real five-section summary requires a different shared scroll position than the three-section illustration; compact headings, existing charcoal tokens/Lucide, truthful destination copy and the existing brand are approved spec differences, not pixel-clone claims. The input uses the established composer slot, with mic separated from explicit Send.
- Fonts/typography: existing system/Thai fallback, readable multiline proposals and secondary copy. Spacing/layout: recording controls wrap; review occupies a full-width row inside the composer, with editable text and distinct Insert/Discard. Color/tokens: existing light/dark surfaces, accent focus ring, red only during recording, disabled Send distinct from editable draft. Assets: crisp Lucide Mic/Square icons; no raster artwork, fake logo or missing imagery. Copy/content: bilingual question/review/pause/privacy/error states, synthetic text explicitly labeled. Full-resolution captures show controls clearly; no additional detail crop is needed.
- Default-adapter packaged QA: startup/welcome has no mic prompt or capture. With no installed speech model, explicit mic click shows setup guidance without asking OS permission, preserves the typed question and keeps Send available: `/private/tmp/velo-mic-missing-model-light-en.png`. Both localized `InfoPlist.strings` exist under actual bundle `Contents/Resources/{en,th}.lproj`, parse with `plutil`, and the merged plist has only microphone usage. Version remains 0.8.0, plist minimum and binary `LC_BUILD_VERSION` remain 11.0.
- Synthetic native acceptance uses the explicit default-off `dictation-qa` build, whose adapters are under `src-tauri/tests/support/` and reject the production identifier. It never requests permission, enumerates a microphone or loads a real model. `/private/tmp/velo-mic-recording-light-en.png`, `/private/tmp/velo-mic-review-light-en.png` and `/private/tmp/velo-mic-insert-light-en.png` prove Recording → Stop → local transcription → editable review → explicit insertion. Editing the main draft while recording, then editing the proposal, appends to the latest draft and returns real focus to the composer. Nothing auto-sends. Opening Settings conceals the composer and cancels active recording; closing it does not restart capture, and both inserted lines remain (`/private/tmp/velo-mic-cancel-mask-dark-th.png`).
- Minimum native window: 1280×800 pixels = 640×400 CSS at 2×, Thai/dark. `/private/tmp/velo-mic-small-recording-dark-th.png` shows real active Stop/status. Opening compact player hides chat and cancels recording; `/private/tmp/velo-mic-small-hidden-cancel.png` shows paused playback, not auto-resume. Reopening retains the typed draft and idle mic. `/private/tmp/velo-mic-small-review-dark-th.png` shows readable proposal and both review actions. Actual Tab from the proposal reaches Insert then Discard; Space discards and returns focus to the unchanged typed draft (`/private/tmp/velo-mic-small-discard-focus.png`). Mounted tests cover native-boundary events, UTF-8 overflow preservation, file/visibility/focus cancellation, stale generations, IME and concurrent edits.
- Through all microphone QA, loopback provider counters remain 7 requests / 6 conversations / 1 deliberate earlier failure / 0 summaries. Cached summary SHA-256 remains `46991c67d0faff26fb587133fbed5fdbac1cd2dfec563adbbde3e0915f678966`. No key/cloud call, real microphone capture, privacy reset, installed-app replacement, push or release.
- Verification gaps are explicit: physical first-run OS consent, real Thai recognition quality, Bluetooth/device runtime, native OS IME candidate interaction and old-macOS execution need user-assisted/device testing. Full Windows application cross-check fails in aws-lc-sys because this Mac lacks Windows SDK `windows.h`; a separate cross-target crate compiling the actual production dictation/CPAL/WASAPI/JobGate/viewport/HWND source passes. This is not a Windows app/runtime pass. Live Ollama, three model-dependent Whisper cases, opt-in model download and signed archive tests remain opt-in skips, not live passes. Existing earlier native workspace/resize/fullscreen evidence remains applicable; dictation does not alter the viewport algorithm.

## Final independent review and one fix pass

- One fresh-context, read-only `gpt-6-astra` whole-branch review found five Important issues, no Critical/Minor. All five were reproduced RED and corrected in one pass: first-use transcript completion/deletion now refreshes local chat metadata without inference or composer teardown; cancelled attempts expose Retry without a duplicate user turn; supplied whole-second labels resolve to exact fractional starts (including short segments and time-query retrieval); oversized segments are split into non-overlapping UTF-8-safe, word-aware slices before ranking, so unique Latin/Thai tail evidence reaches the model within the serialized budget, retaining/deduplicating original intervals; narrow playlist overlays now join the native mask/mic concealment signal while permanently visible desktop sidebars stay exempt.
- The review set aside pre-existing cache-load races and video-mode global shortcut filtering; these were promoted into the same pass because reasonable users still expect correct-file content and focused controls to own keys. Late cache loads cannot replace another file, newer reload or ready event; delayed deletion cannot clear a newly selected file. Native button/select/link/summary activation no longer becomes a global playback action. Focused regressions were observed failing against the previous behavior, then passing; no second review was dispatched.
- Post-fix native 640×400 Thai/dark QA: `/private/tmp/velo-mic-playlist-cancel-fixed.png` shows the mic disabled/cancelled while the compact playlist covers it. `/private/tmp/velo-mic-playlist-reopen-fixed.png` shows the unchanged typed question, idle mic, and no automatic restart after closing. `/private/tmp/velo-chat-stop-retry-fixed.png` records Stop → Retry: one user message, an excluded stopped partial plus the new response, and the newer unsent draft retained. Loopback counts rise from 7 to 9 only for the explicit Send/Retry; summaries remain zero and the cached summary hash is unchanged. These are post-fix interaction captures, not new source-fidelity differences; the combined source comparison and unchanged typography/token/icon review above still apply.
- Final verification: all six requested host commands, `cargo fmt --check`, frontend **205/205**, version consistency **0.8.0**, six release-manifest checks and whitespace pass. The default-feature Apple Silicon production bundle is rebuilt after the fix pass; QA variants never replace `/Applications/Velo.app`. Six opt-in cases are explicitly skipped: three real Whisper cases, live Ollama, model download and signed release archive. Physical consent/recognition/device behavior, full Windows app integration, OS IME candidates and old-macOS execution remain outstanding acceptance gates, not inferred successes.
- Deferred minors: none raised by the independent reviewer. No push, PR, merge, tag or release.

final result: passed

## Conversational answer UI — 21 October 2026

- Scope: the approved focused chat refresh, not a new workspace or content-generation flow. The user's light-theme screenshot shows a nearly full-width gray question and flat assistant text with visible Markdown markers. The new presentation deliberately uses content-sized right-aligned blue-tinted questions, a left-aligned Velo assistant identity, a subtly raised answer surface, a 58rem reading column and more space between turns. Existing system/Thai fonts, Lucide icons, theme tokens, composer, microphone behavior and source contracts remain unchanged.
- Native evidence: `/private/tmp/velo-chat-ui-light-turns.png` and `/private/tmp/velo-chat-ui-light-th.png` show Thai/light conversation and complete answer; `/private/tmp/velo-chat-ui-dark-en.png` shows the same conversation in English/dark. Large viewport is 1440×1024 CSS / 2880×2048 pixels. This is an approved design change rather than a pixel clone; synthetic content and scroll position differ from the user's screenshot. Questions size to their content, assistant headings/bold are legible without raw markers, borders/shadows are restrained, and metadata/prompt review remain secondary.
- Minimum native viewport is 640×400 CSS / 1280×800 pixels: `/private/tmp/velo-chat-ui-small-turns-dark-en.png`, `/private/tmp/velo-chat-ui-small-answer-dark-en.png` and `/private/tmp/velo-chat-ui-small-composer-dark-en.png`. Turn labels, Thai wrapping, citations, prompt content and mic/send controls remain within the reader; long answers scroll vertically. No actionable P0/P1/P2 visual findings in this pass.
- An isolated application identifier and loopback-only synthetic provider were used. Two explicit questions produced two streamed replies; no real key, paid model, physical microphone or user recording was used. `/private/tmp/velo-chat-ui-streaming.png` records the partial reply. The active-status accessibility announcement continues while partial text is present and disappears on completion. Native citation seek was checked while paused at **00:37 / 03:44**; playback visibility remains an independent existing control. Prompt review opens the exact fenced draft, and Cancel leaves saved instructions unchanged. Missing-model mic recovery preserves a typed draft and does not auto-send.
- All six requested host checks pass: `cargo fmt`, warning-denying `cargo clippy --all-targets -- -D warnings`, `cargo test` with the synthetic non-silent WAV, `npm run typecheck`, `npm run lint` and `npx vitest run` (**210/210**, 23 files). Additional format check, version consistency (still 0.8.0), release-manifest checks (**6/6**) and diff whitespace check pass. Five new behavior regressions were first observed RED, then GREEN: safe heading/bold rendering, incomplete streamed markers, literal fenced prompts, live status, and formatted citation/timestamp controls.
- Safety/coverage: Vue text interpolation is retained; no generated HTML or external link is executed. Mounted coverage checks exact fractional seek through nested bold content and shared timestamp hiding. Existing dictation, retry, Stop and IME tests also pass. Six opt-in Rust cases remain skips: three real Whisper cases, live Ollama, model download and signed archive. Physical microphone/OS consent, live provider quality, Windows runtime, OS IME candidate interaction and old-macOS runtime are not newly verified by this UI pass.
- Large and minimum-window Apple Silicon QA bundles build with default production features, not a synthetic microphone adapter. The deliverable is a separately named, unopened `Velo Chat Preview.app`; the original `Velo.app` bundle is not overwritten. No installed-app replacement, push, PR, merge, tag or release.
- The user chose not to restart the original app. Only the isolated QA application was opened/quit by this pass. The final process check found no original Velo process; it was not relaunched, and continued preservation of an already-running session is not claimed.

Chat UI result: passed; device/live-service limits are recorded above.

## Selectable summary formats — 21 October 2026

- Approved extension to the existing summary flow: Meeting summary (default), Key points, and My own prompt. Custom instructions do not inherit preset headings, bullets or section order. Meetings omit unsupported sections rather than inventing outcomes. All modes retain source grounding, language selection and citations. Existing uncommitted chat-UI changes are preserved; no new provider, dependency, permission or transport was added.
- Backward compatibility: older settings retain model, instructions and language while defaulting to meetings. Old cached Markdown loads unchanged. New results store their own format, and rendering follows that stored format rather than the next-run setting. Only meeting results receive outcome-priority ordering and folded discussion; key-point/custom headings remain in original order and visible.
- Test-first evidence: `/private/tmp/velo-summary-formats-red-frontend.log` records five intended failures for format persistence, bilingual choices, custom ordering and folding. `/private/tmp/velo-summary-formats-red-rust.log` records five intended failures for prompt isolation and settings/cache compatibility. Subsequent focused runs pass. Mounted controls additionally check atomic Save, failed-save rollback, Cancel/reopening, retained draft edits and format-appropriate examples. Wire tests exercise all three formats in single/map/reduce stages through Ollama, OpenAI and DeepSeek (27 combinations); no real provider is involved.
- Native QA uses only the isolated `com.velo.summary-controls-qa` app, synthetic media and loopback port 49997. Saving Key points initially leaves provider counts at zero and the original summary SHA-256 at `46991c67d0faff26fb587133fbed5fdbac1cd2dfec563adbbde3e0915f678966`. The following three explicit runs generate the matching key-point, plain-paragraph custom and meeting outputs. Screenshots: `/private/tmp/velo-summary-formats-key-points.png`, `/private/tmp/velo-summary-formats-custom-paragraph.png`, `/private/tmp/velo-summary-formats-meeting.png`, and `/private/tmp/velo-summary-formats-editor-custom.png`. Large viewport: 1440×1024 CSS / 2880×2048 pixels, English/dark.
- Minimum viewport: 640×400 CSS / 1280×800 pixels, Thai/light. First inspection found a P2: autofocus on the textarea scrolled the newly added format selector out of view. The regression fails with TEXTAREA instead of SELECT, then passes after choosing initial focus by entry path. `/private/tmp/velo-summary-formats-small-focus-fixed.png` confirms the selector is visible on normal open. Reviewing a chat proposal still focuses its draft. `/private/tmp/velo-summary-formats-small-save-visible.png` confirms editing and Save remain reachable by scrolling; Tab reaches Save and Space submits without a playback shortcut.
- Actual native map-reduce acceptance uses a temporarily expanded synthetic transcript with an 8192-token QA context. Two map requests and one reduce produce the requested custom heading order: first สาระหลัก, then ประเด็นที่คุยกัน without folding, then ภาพรวม without promotion. `/private/tmp/velo-summary-formats-small-custom-order.png` and native accessibility state confirm order/visibility. The cache records `format: custom`. Timestamp off/on preserves the content; Escape after an unsaved format change restores the saved custom choice on reopening. Final mock counts: 6 requests, 2 map passes, 4 final answers, 0 old-heading/reminder violations. The original synthetic transcript is restored exactly after QA.
- Final verification: all six requested commands, `cargo fmt --check`, version consistency (0.8.0), six release-script checks and whitespace checks pass; frontend **221/221**, 24 files. One earlier full run failed `extracts_16khz_mono_wav` with no extracted audio despite a valid synthetic WAV. The isolated reproduction passed, followed by complete passing reruns; no extraction code was changed and no unproven root cause is claimed. Six opt-in cases remain skips: three real Whisper cases, live Ollama, model download and signed archive.
- QA is a functional/visual check, not a claim about live model quality, physical mic/OS consent, OS IME candidates, Windows runtime or old-macOS execution. Existing light/dark tokens, system/Thai typography, Lucide icons, escaping and clipboard source Markdown contracts remain unchanged. No actionable P0/P1/P2 findings remain in this pass.
- Large and fixed minimum-window QA bundles build. The deliverable is a separately named `Velo Summary Formats Preview.app`, kept unopened. The user's running `Velo Chat Preview.app` is not quit, restarted or overwritten. No version bump, push, PR, merge, tag, installed-app replacement or release.

Summary format result: passed; live-service and platform limits are recorded above.

## Source fidelity and confirmed spellings — 21 October 2026

- Approved bounded change to the existing summariser, not an ASR rewrite. Single/map/reduce prompts now share source-fidelity rules after user steering and short trailing reminders. Correct phonetic spellings only with source/user confirmation and clear local context; vocabulary is not evidence of new facts. Preserve uncertainty, scope and event conditions. A configuration default is not a delivery order; starting and completing a review are distinct triggers. Raw transcripts and cached source timestamps are not modified.
- Test-first evidence: `/private/tmp/velo-grounding-red-unit.log` has three intended contract failures before the fix. `/private/tmp/velo-grounding-red-wire-mutation.log` proves removing the default/order guard fails the emitted request assertion. First sandbox wire attempts fail to bind loopback, an environment restriction, not a product regression; allowed reruns pass. The first live custom paragraph compressed a completion condition, so `/private/tmp/velo-grounding-triggers-red.log` records the added trigger/unclear-term failures before the second correction.
- Final whole-branch gate passes: `/private/tmp/velo-grounding-final-verification.log`, exit 0 (session 59943). All six requested commands, format check, version consistency 0.8.0, six release-script checks and whitespace pass. Frontend 221/221, library 56 tests; the wire matrix exercises 3 providers × 3 formats × 3 stages. Six opt-in early-return cases remain skips: three real Whisper, live Ollama, model download and signed archive. These are not claimed as live recognition/provider/archive passes.
- Default-feature Apple Silicon bundle `Velo Source Fidelity Preview.app` builds successfully; `/private/tmp/velo-grounding-final-preview-build.log`, exit 0 (session 47648). The earlier intermediate `Velo Grounded Summary Preview.app` is not the final code. Original user app sessions were not quit/replaced. A new-build Keychain read waits for OS authorization; a read-only stack sample identifies that boundary and the user unlocks it themselves. No credential is exported, printed or stored in a new location.
- User-authorized live DeepSeek acceptance reuses the existing 3:44 recording and 53 cached segments. After the second fix, custom, key-point and meeting jobs each complete in the actual final app, preserving their 0/2/5-heading formats. The custom request is the same one-paragraph, at-most-150-word instruction used before; the final result has approximately 149 Thai words. All three final samples remove the phonetic Default error and the old unsupported development-order assertion, and retain the review-completion condition. Owners/deadlines are not invented. The raw transcript SHA-256 remains `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`. Original format, instructions, vocabulary and provider preferences are restored without another model request.
- Results, untouched model outputs, original-summary backup, validation script and honest limitations are local only in `/private/tmp/velo-grounded-summaries.8S8nRM/`. Automated acceptance checks supplement manual source review; they do not establish future model reliability. Remaining live-quality limits: two meeting citation labels and one custom label are not exact source starts; the key-point overview omits citations and repeats one point; the meeting output's open-question treatment of a group-name transition is inconsistent with its discussion, and the custom wording about not using stored data yet is stronger than the transcript. The report explicitly flags these for human source verification. No claim of perfect grounding, corrected raw ASR, physical mic, Windows or old-macOS runtime is made.
- No dependency, credential-policy, transport, permission, version, installed-app, push, PR, merge, tag or release change. Existing uncommitted summary-format/chat UI work remains intact.

Source-fidelity result: targeted Default/order/completion regressions improved in all three live samples; residual model-quality limits remain disclosed, not treated as passing factual guarantees.

## Bounded source review and exact references — 21 October 2026

- Approved extension: drafts and per-excerpt map notes receive a source-review pass before publication. Final reduction uses checked notes. Each stage allows one review and at most one repair; empty/unsupported output, missing citations, unknown references, context overflow and cancellation fail closed. Unchecked drafts do not stream into the reader. No new dependency, provider, permission, credential store or persisted summary schema is introduced.
- Native testing uncovered an existing Regenerate problem: it deleted the old cache before starting inference, bypassing the new backend save boundary. The exact original backup was restored (SHA-256 `f34b49aae80195faf39ed28365997c793b63caacb4bc3d7de714fa11a47d900a`), and the button now leaves the old result intact. `/private/tmp/velo-regenerate-preservation-red.log` records its intended deletion-call regression before the fix. Later real validation failures retain the preceding custom cache byte-for-byte (`00b75c70ec33da666ded66217817207a352766723b6dd00ddc4d5afbd84a6279`) and display an alert alongside it.
- Actual model trials exposed bare organizational labels being treated as uncited facts, legacy meeting instructions overriding key-point structure, and nested task headings being detached by section sorting. Explicit heading/format contracts and nested-section rendering address these. Red evidence is in `/private/tmp/velo-review-headings-red.log`, `/private/tmp/velo-review-format-priority-red.log`, and `/private/tmp/velo-review-nested-headings-red.log`. The native accessibility tree now places task subheadings at level 4 under the level-3 parent, without a false empty-task placeholder.
- Clock-copy instructions still produced the nonexistent `[00:31]` in both review and repair; bounded failures were retained rather than rounded or saved. Model-facing chunks now use stable original-segment IDs such as `[S000007]`, including across overlaps. Validated IDs resolve by exact lookup to the existing clock-cited Markdown. `/private/tmp/velo-source-ids-red.log` records the pipeline's inability to accept the ID-backed test before implementation; the real loopback tests then pass. Unknown-ID and exact-lookup tests supplement the earlier timestamp-membership cases. The source transcript remains untouched.
- Latest full gate is `/private/tmp/velo-reviewed-final-verification.log`, exit 0 (session 97033). All six requested commands, format check, version consistency **0.8.0**, six release-script checks and whitespace pass. Frontend **229/229**, 25 files; Rust library **62**; six new pipeline integrations cover all formats, checked-map reduction, bounded repair, cancellation, context overflow and old-cache preservation. Six opt-in early-return cases remain skips: three real Whisper, live Ollama, model download and signed archive. These are not counted as live recognition/archive successes.
- The default-feature Apple Silicon bundle `Velo Source Reference Preview.app` builds successfully (`/private/tmp/velo-source-reference-preview-build.log`, session 84542, exit 0). It is the final build; the other named review previews are intermediate builds. Existing app sessions are not quit or replaced, and `/Applications/Velo.app` is untouched. No key is exported or logged.
- User-authorized real DeepSeek acceptance on the same 3:44 / 53-segment recording finishes all three formats in the final build: meeting **5 headings / 59 reference occurrences**, key points **2 / 38**, custom **0 / 22**. The custom instruction is the identical one-paragraph/no-list/at-most-150-word prompt; the result has about **137** Thai word segments. Every final reference label belongs to the original source set, and no internal ID leaks into saved content. Both raw outputs and the check script are in `/private/tmp/velo-reviewed-summaries.rPr1Sh/`; pre-ID samples are preserved separately, not presented as final-build evidence.
- Native citation navigation while paused reaches **00:30 / 03:44**. Mounted reader tests assert the exact **30.90** value (rather than 30.0 or a nearby interval); the backend uses `absolute+exact`. Unknown, ambiguous, noncanonical, absent and other-file source labels cannot seek. Timestamp off/on hides/restores controls without inference or cache changes. Player is returned to hidden/paused and discussion to folded. Original summary preferences, vocabulary and destination are restored. Transcript SHA-256 is still `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`.
- Semantic acceptance is **partial, not a factual guarantee**. All three final samples preserve the review-completion trigger, no longer use the phonetic Default error as a name/development order, and omit the earlier unsupported not-used-now assertion. The resolved group-name transition is no longer an open question. Remaining caveats are explicitly recorded in `comparison.txt`: a repeated key-point sentence may imply group mode already exists; the meeting's future-data-use open question is inferred from unspecified detail rather than an explicit unanswered question; unclear ASR terminology still needs human listening. Valid IDs establish source location, not semantic entailment. No claim of perfect source fidelity or whole-recording audio verification is made.
- No commit, push, PR, merge, tag, release, version bump or installed-app replacement. Existing uncommitted chat/format/UI work is retained.

Result: automated checks, source-reference integrity and native navigation pass; live semantic quality retains the disclosed limitations.

## Thai reading typography — 2026-10-21

- User-approved bounded change: bundled looped Noto Sans Thai, reading body 16px/1.8, headings weight 600, existing monospace time/code unchanged. No Evidence Ledger implementation, content rewrite, API call, cache mutation, installed-app replacement, push, or release.
- Real Vue summary, transcript, chat-message and composer components were rendered with synthetic Thai/English in a local browser fixture. Baseline summary computed system sans at 18px/33.3px. After the change the font-face reports `Noto Sans Thai: loaded (100 900)`; reading content computes 16px/28.8px and headings 600. Transcript timestamps and chat code still compute ui-monospace. Tailwind's dev process needed a restart to pick up the font-family config; the fresh session and production CSS both use the new family.
- Inspected stacked Thai marks in light/dark at 1080×720, plus 360px-wide transcript, summary and chat panels. No horizontal prose overflow in the sampled content. Images `/private/tmp/velo-cache-audit.Hz1fOp/thai-summary-light.jpg` and `/private/tmp/velo-cache-audit.Hz1fOp/thai-chat-dark.jpg` show synthetic fixture content, not new summaries of the user's recording. Native installed-app rendering is not claimed in this pass.
- Fresh gate: cargo fmt, fmt --check, warning-denying clippy, cargo test, npm typecheck, lint, Vitest **229/229 in 25 files**, and production frontend build all exit 0. Rust used the local Whisper model and non-silent test audio, so the real extraction/inference cases ran. Explicit DeepSeek tests remain ignored; network download, live Ollama and final release-archive tests retain their opt-in early returns. Font and complete OFL license are included in dist/fonts and byte-match the source assets.
- Separate cache investigation re-transcribed the actual recording offline without replacing its cache. Matching content and source positions support the cache/file association; phonetic ASR errors remain and no historical content hash exists. Private details and unchanged user-data hashes are in `/private/tmp/velo-cache-audit.Hz1fOp/report.txt`, outside the repository. This does not certify the pending summary-fidelity work as complete.

## Evidence Ledger — 21 October 2026 (acceptance failed, unshipped)

- User approved Native execution, additional DeepSeek calls for the same recording, and three consecutive complete three-format rounds. The implementation replaces the prose draft/review/audit path with extraction, field verification, ID-only layout and deterministic composition. Exact job-scoped raw anchors, bounded requests, atomic replacement, additive omission metadata and bilingual progress/error copy are implemented. No automatic regeneration or certification is implied.
- RED→GREEN evidence and the initial dirty-work backups are in `.superpowers/sdd/2026-10-21-evidence-ledger/`. All prior chat/dictation/format/font changes are preserved on the existing branch. The fresh independent whole-worktree review found no Critical issue, but identified lost question state, cascading spelling edits, cross-file terminal-event ownership and meaningful coverage gaps. One fix pass adds localized question states, requires a selected linked answer, applies spelling edits against original UTF-8 positions, rejects overlapping/crossing edits, and ignores another file's terminal events. Literal all-format condition/qualifier/planned-scope, distant-answer, follow-up-versus-repair exhaustion and event-driven regressions supplement the original suite. Synthetic model verdicts are not entailment tests.
- Final six-command gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run typecheck`, `npm run lint`, `npx vitest run` pass. Also format check, QA-feature all-target clippy, version consistency **0.8.0**, Node acceptance-checker **4/4**, and whitespace checks pass. Frontend **241/241, 26 files**. Rust library **93 passed / 1 ignored**; default integration targets report **97 passed rows**, including three opt-in early-return rows. Local Whisper extraction, inference and cancellation actually run using the installed model and synthetic non-silent WAV. The network model-download, live Ollama and final signed-release-archive checks are still opt-in early returns; the obsolete live audit test is ignored. The new DeepSeek harness is feature-gated and ignored unless explicitly invoked.
- Private acceptance directory: `/private/tmp/velo-ledger-acceptance.bmffnd/`. Original build attempt uses **7 logical model calls**, all three formats reject the schema placeholder job token. That protocol bug is reproduced RED then repaired GREEN, and all gates/bundle rerun before restarting. The protocol-fixed build `b3247b67010b66ef5ee0b486246a65968a31a5788016aa80eb169853e831208c` uses **9 further logical calls**: meeting2/ceiling4 fails extraction, key-points5/ceiling8 produces an isolated but semantically failing output, custom2/ceiling4 fails extraction. Total16 logical calls, not nine successful summaries; provider billing and physical retry counts are not inferred.
- Manual comparison covers the original 53 raw source segments, candidate fields, verification reports, admitted ledger, layouts and output. Key-points admits `existing` scope from design discussion and an unresolved state from a bare question, leaves Default noisy, and misses required completion/backend/shared-box/reviewer content. Meeting's repair uses question strings rather than anchor objects; custom omits required qualifier arrays in both packets. Both fail closed. No custom paragraph exists to measure; no fabricated word count is reported. The Node checker records v22.14.0/ICU76.1, `consecutive:0`, `passed:false`, exit1 as expected. **Accepted complete rounds remain 0/3. Rounds2–3 are not run after the known meaning error.** Further live calls stop under the approved plan rather than relaxing or cherry-picking the checklist.
- The later deterministic review fixes change the final code identity to `40fd9eec8f954dc551a28a41330c2283916e2694e46c95fd82ea16a2a299e716`. There is **no successful live acceptance run on that final build**. A passing gate/build cannot inherit acceptance from earlier code or convert failed samples into passes.
- The final default-feature Apple Silicon bundle `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/Velo Evidence Ledger Preview.app` builds successfully, version0.8.0, and is kept unopened. `/private/tmp/velo-ledger-reviewed-final-build.log` records exit0. The existing `STATIC_VCRUNTIME` deprecation warning is unchanged; it is not a compile failure. The QA-feature observer equivalence test also passes on final code; the shipped-default build does not depend on that feature.
- Real SummaryTab visual QA uses a separate offline browser fixture, not production state or a claimed native run. Thai/English accessibility copy, three step4/4 labels, omission and unsupported-layout messages beside old content, light/dark and a 360px panel were inspected. Images `ui-th-light.png` and `ui-th-dark-360.png` in the private directory show synthetic content. A later HMR reset is honestly captured as `ui-after-hmr-th-light.png`, not English/dark proof. The temporary tab/server are closed. Native rendering of these new states is still unverified; the user's running Thai Font Preview was not closed/replaced.
- Production transcript, summary and settings hashes match the pre-test snapshot. The source transcript remains `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`; the complete unchanged hashes, failed checklists, timings and raw private artifacts are in the private report. No key/password is exported or logged. No version bump, commit, push, PR, merge, tag, installed-app replacement or release.

Result: local implementation and automated gates exist, but task6/semantic acceptance is **incomplete and blocking**. Extraction coverage and classification require another bounded design decision before a fresh unchanged-build 3×3 sequence. General model reliability, physical microphone, Windows/old-macOS execution and new-state native visual QA are not established here.

## Simpler source-grounded summaries — 21 October 2026 (live acceptance pending)

- The approved Native replacement removes the full field taxonomy, special content variants, answer graph and spelling-edit chain. One runner extracts small source-backed points, selects whole IDs into a layout, reviews each visible occurrence with its heading, then composes and saves locally. A supported overview occurrence cannot certify the same point under a decision heading. The original nine-job semantic acceptance gate is unchanged.
- Tasks1–4 have RED→GREEN evidence and recoverable uncommitted checkpoints in `.superpowers/sdd/2026-10-21-simple-grounded-summaries/`. Existing dirty chat/microphone/font/player work is retained. No second production runtime or semantic retry loop is added. Synthetic verdicts exercise enforcement, not factual entailment.
- Fresh checks pass: cargo fmt, fmt --check, default and QA-feature all-target Clippy with `-D warnings`, cargo test, npm typecheck/lint, Vitest **247/247 in 26 files**, version consistency **0.8.0**, and whitespace. Rust library **86 pass/1 ignored**; default integration targets report **92 passing rows**. All four real transcript-pipeline cases run using the installed Whisper model and synthetic non-silent WAV (39.88s). Model download, live Ollama and signed final archive retain three opt-in early-return rows; the historical live audit is ignored. QA pipeline **8/8**, Node checker **8/8**, all Node release/checker tests **14/14**. Final gate sessions37064/39399 and `task-5-qa.log` retain the results.
- Observer/default jobs issue identical requests after normalizing job identity and compose identical text. Artifact metadata is allowlisted before writes; dummy credential sentinels and credential/header/JobInput properties are absent. E/V identities use source/occurrence ID sets, not feedback or fetched context. The checker resets on source/settings/build changes, gaps or missing/failed criteria. Escaped literal source clocks/headings are not mistaken for application references or custom-layout blocks. Frontend source is unchanged in this task; existing event/cancel/late-file/exact-seek/copy regressions pass.
- The default-feature Apple Silicon bundle `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/Velo Simple Grounding Preview.app` builds successfully, version0.8.0 (`task-5-build.log`, session74935). It remains unopened and uninstalled. The existing STATIC_VCRUNTIME deprecation is not a build failure. The running Thai Font Preview PID89577/binary and installed app are untouched.
- Private acceptance root `/private/tmp/velo-simple-grounding.h2oCd0/` contains verified input provenance and frozen settings. The original cache remains **53 segments**, SHA-256 `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`, with the offered recording duration224.433333s. Production source/settings/summary hashes match before acceptance. Final runtime/test/build/assets identity is `1fd73b6309c024039431ff2286c551096968a6ce2e39a920617809fd814ca739`; frozen-settings identity `63553b5452e281c3a3fa5381b6355130123106700a1a1d51230a1c0c356a7b70`. The final manifest includes263 files, not just HEAD. Historical trials cannot count toward this build.
- At original dispatch, the explicitly opted-in DeepSeek round1 process (session81035) waited for the user's macOS Keychain authorization before its first model request. Its eventual failed result is recorded below. The user was asked to authorize on the machine, never to send a key/password in chat. Do not label an ignored test or a valid schema as live acceptance.
- No push, PR, merge, tag, release, version bump, production-cache/settings write or installed-app replacement. Private raw results/backups are retained for recovery. Physical microphone accuracy, Windows/old-macOS execution and native visual QA of new states are not established by this pass.

Update after Keychain authorization:

- The first live attempt actually ran **6 logical calls**, two extraction calls per format (E1/V0/ceiling4 each). All three failed closed before layout/review/composition. Required source quotations were paraphrased out of point text even after generic repair feedback. No output exists for semantic or custom-word-count sign-off; the recorded checker reports consecutive0/passedfalse. Failed artifacts remain in the first acceptance root.
- A bounded protocol fix now reports `required_quote_missing` with the point/phrase index and exact offending source quotation, instructing the model to check every required phrase without deleting protection. Extraction reserves source-packet-sized repair space before HTTP. No quote validation, call ceiling, retry count or acceptance criterion is relaxed. Synthetic feedback and budget cases are RED→GREEN.
- Fresh post-fix six-command gates, fmt check, default/QA Clippy, version/whitespace and Node14/14 pass. Rust library87 pass/1 ignored; default integrations93 reported rows, with the same3 opt-in early returns and all4 real Whisper cases exercised33.75s. Frontend247/247; QA pipeline9/9. Logs: `repair-final-rust.log`, `repair-final-frontend.log`. The separate default preview rebuild passes (`repair-final-build.log`), remains unopened, and the running preview binary still matches.
- Fresh repaired-build acceptance root `/private/tmp/velo-simple-grounding-repair.KZO648/`; runtime/test/build/assets identity `e2e79b1952264c417a71a7a864bbc24b7f46dc6416bfe2733d6e434c4446a770`, same frozen source/settings. Round1 initially waited for macOS Keychain authorization after recompilation; its final result is recorded below. Earlier failed calls cannot count toward this build. No repaired-build successful round, rounds2–3 or final independent review is claimed.

### Repaired-build result after authorization — 22 October 2026

- Session36887 finished exit101. All three formats failed extraction after their one bounded repair: meeting E1/V0, 2 requests/ceiling4, 5707ms; key-points E2/V0, 4 requests/ceiling6, 12359ms; custom E1/V0, 2 requests/ceiling4, 5456ms. These eight logical requests exclude the overnight authorization wait; they are not a provider billing/retry count. The harness's overall elapsed time includes that wait.
- Offline comparison of actual supplied source rows and responses confirms exact anchors, but required phrases are missing verbatim from point text after repair (meeting five, key-points second packet two, custom two). The first key-points packet did pass on repair; the second initially returned eleven points instead of at most six. The specific feedback did not make the extraction contract reliable. Drafts also contain questionable interpretations, but none reached the placement reviewer or reader; there is no final text to certify.
- No layout, review or composition ran; no new summary was saved. Custom shape/word count and semantic timestamp relevance cannot be evaluated without final output. The private `round-1/acceptance.json` records all unmet criteria as false; the unchanged checker exits1 with consecutive0/passedfalse, Nodev22.14.0/ICU76.1. Rounds2–3 were not started, and no further cloud requests are being used to search for a lucky pass.
- All263 frozen code-file digests still match. Original transcript and production-summary hashes, frozen QA settings, summary configuration, running-preview binary and installed Info.plist match. The production settings file has changed only at `updates.last_checked_at` while the app remained running; the harness never writes that file, and no production preference is restored to erase the updater's timestamp.
- Fresh six-command local gates pass, together with fmt check, default/QA warning-denying Clippy, QA pipeline9/9 and review5/5, all Node release/checker tests14/14, version0.8.0 and whitespace. Frontend247/247; Rust library87 pass/1 ignored, default integrations93 reported passing rows including the same3 opt-in early returns. All4 real Whisper cases ran34.68s. Logs are `post-unlock-rust-allowed.log` and `post-unlock-frontend.log`; the first sandboxed Rust run was denied local TCP bind and is retained as environmental failure evidence in `post-unlock-rust.log`, not counted as green. No product code was changed during this diagnosis. Final independent review remains pending because task5 acceptance is blocked. A revised extraction contract needs a bounded design decision before implementation or another unchanged-build sequence.

Result: task4 and automated task5 gates/build pass; original-file **3×3 semantic acceptance remains incomplete**, not complete.

## Source-bound extraction amendment — 22 October 2026

- User approved the written scoped spec/Native plan and additional DeepSeek calls. Tasks1–2 are implemented with observed RED→GREEN regressions: typed extraction parts, exact raw-byte assembly, local exactly-once slots, strict/unique supplied anchors and evidence coverage, bounded category feedback/reservations, and the single production extraction seam. Downstream whole-point layout, heading-aware review, composition, cache and public schemas remain unchanged. Existing dirty chat/microphone/font/player work is preserved, with uncommitted recoverable checkpoints; no push/release or app replacement.
- Fresh six-command gate passes: cargo fmt, warning-denying all-target Clippy, cargo test, npm typecheck/lint and full Vitest **247/247 in26 files**. Also fmt check, QA all-target Clippy, QA pipeline11/11+review6/6, all Node release/checker14/14, version0.8.0 and whitespace checks pass. Default Rust reports95 library passes/1 ignored and96 integration passing rows; three rows are explicitly opt-in early returns (model download, live Ollama, signed archive), not exercised live passes. All4 real Whisper extraction/inference/cancellation/timeline cases ran with the installed model and synthetic non-silent WAV,39.84s. Logs are `final-rust.log` and `final-frontend.log` in this plan's ignored workspace.
- Separate default-feature Apple Silicon bundle passes, session38537, optimized build41.77s; `Velo Source Bound Preview.app` stays unopened/uninstalled. Existing STATIC_VCRUNTIME deprecation remains unrelated. The pre-existing Thai Font Preview process89577 is still running; its executable and `/Applications/Velo.app/Contents/Info.plist` hash-match before/after. This pass makes no native UI or device/Windows/Intel claim.
- Private root `/private/tmp/velo-source-bound.DfXIwm/`. Frozen runtime/test/build/assets identity **5c9574d42d66a2e4d5e0f2176a0094a2c8571d385847f2252f7545cdd4dbe72a**,264 nonignored files; frozen settings **8db2d9a97ddd6b9ad0b237e3d0fba32cacf5b798253ab4b30e56039960ab9bc0**. The same53-segment source SHA-256 remains **47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15**, offered recording224.433333s, saved DeepSeek/deepseek-flash/context32768. The saved key was consumed only by Rust Keychain access, not exported to shell/log/artifacts. Existing meeting/key-point focus is frozen; custom uses the exact approved single-paragraph/150-word prompt.
- Session32761 finishes exit101 after all three formats in round1: meeting E2/V0,4 logical requests/ceiling6,11625ms; key-points E2/V0,3/ceiling6,9916ms; custom E6/V4,14/ceiling22,27669ms. Total **21 logical requests**, not a physical-retry/billing count. Meeting's initial first-packet anchor has two raw matches; its repair passes. The second packet initially sends an array instead of an integer reference, then its repair declares a required source quotation not covered by its evidence. Key-points' second packet first cites a repeated phrase and then repairs to another ambiguous required quotation. Both fail closed before layout/review/composition; neither produces a summary.
- Custom admits30 canonical points. Offline object-order-independent comparison confirms every canonical point exactly matches its accepted wire parts and protected raw phrases. This does not certify prose fluency: surrounding literals duplicate phrases such as Default/ก่อน and คน, and retain noisy recognition. Layout selects13 occurrences in three paragraphs. It never selects P000012 (the completed-review removal condition/question from S000020–21), nor the related P000013/P000014. The occurrence reviewer cannot recover omitted points; this is not a false reviewer verdict on the missing condition. Four final review reports support12 selected occurrences and reject one uncertain future-use fragment; checking includes two bounded structural repairs. No extra rewrite occurs.
- The custom result is saved **only in the isolated QA store**, then the live harness fails its existing single-paragraph assertion. Independently removing app citations and segmenting with Intl.Segmenter('th',word) yields **227 word-like units in3 paragraphs**, Nodev22.14.0/ICU76.1. Requested person/group choice, an explicit person Default and backend read/unread retention appear, but the mandatory completion condition is absent. No full semantic-inference or source-time sign-off is claimed from valid clocks, and noisy quoted text is not a correctness certificate. All original passages, wire outputs, canonical points, placements, review reports and final text were inspected offline for these failures; no difficult required topic is waived.
- Private `round-1/acceptance.json` records known passes/failures and uncertified criteria as null. The unchanged checker exits1, **consecutive0/passedfalse**. Rounds2–3 are not started; older trials do not count. No additional unchanged-build cloud retries or prompt-only experiments are used to manufacture a pass. Product changes beyond this extraction-only amendment need a bounded scope decision, then fresh gates/default bundle/new unchanged-build3×3.
- `round-1/diagnostic.json` confirms all264 runtime-file identity bytes, frozen settings, original source, production summary/settings, running binary and installed Info.plist remain unchanged. No production preference restoration or new app launch occurred. Failed artifacts and isolated custom output remain available locally; recovery archives/workspace are retained rather than deleted while work is uncommitted.

Result: amendment tasks1–2 and task3 automated gates/default bundle pass. **Task3 original-file3×3 acceptance and final independent review remain incomplete.** This is not a shipped or source-fidelity-approved fix.

## Limited throwaway feasibility spike — 22 October 2026

- User approved one original-file three-format round, extra DeepSeek calls and stopping on the same failures; no push/release or product adoption. Private `/private/tmp/velo-summary-spike.BxDokE/` contains a disposable standalone prototype and REPORT.md, not another production runner/spec/implementation plan. It reuses the actual public Rust transport and Keychain access, keeping the key inside Rust. No product code/interface/dependency changes are made.
- The prototype replaces model-copied quotations with supplied whole-source IDs, requires model-marked essential points to survive selection, and would review missing topics against all raw sources. Host custom guards require one paragraph/no headings/list/at most150 Thai word-like units. Model flags and exact IDs cannot prove semantic completeness or entailment. Seven grouped offline guards were observed RED→GREEN (7/7), covering schema/source IDs, omission, fractional time targets, escaping, custom shape/length and rejection of a failed semantic report. They are not live model-quality proofs.
- Frozen prototype code identity a6dc1b20bd2772ec393df0715d89e3a8dd617ecd3dbe1b92c53eb9dd7ec33fcb; frozen settings12d824945597c682f52c6cffd519fbcd0da51fd0beae990a37db06188e7e31f6 (pretty serialization of the same summary preferences). Original source remains53 segments/SHA25647d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15, same224.433333s recording and DeepSeek/deepseek-flash/context32768. Full hashes and standalone bridge binary/lock provenance are private; bridge built offline in1m19s.
- One live round, session86292: meeting9 points, key-points10, custom19, all over the prototype's8-point limit. One extraction call each: **3 logical requests total**, provider waits2841/2697/4760ms, no repair/rewrite/second round. The8-point limit was an implementer-chosen experiment constraint, not a user fidelity requirement; its failure does not prove the provider generally unsuitable. All individual point shapes/source IDs are valid in an offline diagnostic, but no over-limit packet is admitted or truncated. Script exit0 means the investigation finished; all `passed_local` flags and `production_acceptance` are false.
- All three stop before format-specific layout, missing-topic review, custom shape or live composition/citations. They are three attempted formats, **not three completed formatted summaries**. All original raw passages and rejected drafts were inspected. Custom still guesses “ให้ดีฟ้าเลือกลุ่มก่อน” from noisy S000002 despite the clear person Default in S000025; uncertain permission interpretations and question-to-statement promotion also remain in drafts. These drafts are rejected before review, so the proposed reviewer has neither passed nor failed a live meaning test. The proof does not establish an end-to-end fix; no general claim that changing models is necessary follows.
- Fresh existing-product checks pass: cargo fmt/check, strict all-target Clippy, cargo test, npm typecheck/lint, Vitest247/247 in26 files, Node14/14, version0.8.0 and whitespace. Rust95 library pass/1 ignored +96 integration rows, including3 explicit opt-in early returns; all4 actual local Whisper cases run38.18s. Sessions93887/16374 exit0. No new app bundle/UI/runtime acceptance is claimed for this standalone probe. preservation-after.json confirms all264 product-file hashes, original transcript/production summary/settings, prior preview executable and installed Info.plist unchanged.

Result: authorized **one-round investigation complete, proof not passed**. Stop further API calls and retain private failed artifacts; do not adopt throwaway code or mark prior product acceptance/review complete. No app launch/replacement, push, PR, merge, tag or release.

## Original-passage selection — 22 October 2026 (local, not accepted)

- User approved extractive original wording, additional API calls and testing
  the original file in all three formats before a preview launch. This is not
  a new successful3×3 sequence. Prior dirty chat/microphone/font work is kept;
  `/private/tmp/velo-extractive.19ZlZJ/before.tar` is a verified recovery archive.
- Observed RED→GREEN: source-ID selection copies whole raw rows into the
  existing canonical quotation/evidence/required point; reviewer source IDs
  resolve to whole raw anchors locally. The model cannot return prose, quotes,
  offsets or spelling edits. Unknown/unsupplied/duplicate/reordered sources,
  legacy keys and wrong jobs fail closed; the supplied row count replaces the
  arbitrary six/eight-point cap. Existing canonical/public/cache contracts,
  heading review, cancellation and the2E+2V+2 ceiling remain.
- First production-path three-format run:31 logical calls. Meeting creates an
  isolated result; key points fails layout after returning meeting headings
  twice, and custom creates one paragraph but241 Thai word-like units, without
  its clearer later default/retention material. No all-format acceptance.
- That observed layout seam is reproduced RED→GREEN: the schema advertises
  only the selected format's headings and sends quotation text once rather
  than repeating it inside evidence/required arrays. A fresh build uses30
  logical calls and creates all three outputs, but custom is313 words. This
  is not accepted merely because the harness exits0: its shape assertion does
  not count words, and manual acceptance still fails the150-word request.
- A local ICU4X2.3.0 dictionary counter and explicit custom-word-ceiling guard
  are added after RED→GREEN tests. Recognized numeric Thai/English ceilings
  constrain whole-point selection, with per-point counts and maximum repair
  envelope reserved before HTTP. Only the existing layout repair slot is
  available; no text truncation or extra prose writer is added. Four ICU
  packages are added to the lockfile; no prior dependency version is bumped.
- Final frozen runtime:172 source/config/test/asset paths, aggregateSHA256
  `ab6307c125cff3e3736440368653ee448cfdb79d682bfb5e86d9b8ed28999ec4`.
  Original transcript SHA256 stays
  `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`.
  The saved focus/settings are frozen privately, without API credentials;
  only Rust Keychain reads the already authorized key. Outputs never replace
  production caches/settings.
- Final original-file run (`length-fixed/`, session57529):32 logical requests,
  meeting12/ceiling24, key-points12/ceiling24, custom8/ceiling14. Both extraction
  and layout ceilings remain enforced. Custom's actual selections total364
  then262 words and are rejected before placement review/composition/save.
  No custom final text exists to certify. These three builds used93 logical
  requests total, not a provider billing or physical retry count.
- Human/source comparison: admitted meeting/key-point quotation text equals
  its original raw rows, and clocks derive from their original starts. Both
  retain choice/person-default, completed-review removal question, backend
  retention, shared-inbox visibility and actual-reviewer material. However,
  meeting also accepts the ambiguous closing S000051 excerpt under actions;
  a verbatim fragment is not evidence of a standalone assigned/requested task.
  Its no-inference check is false. Key points uses neutral headings but remains
  verbose/noisy. `length-fixed/acceptance.json` records the failure honestly;
  no complete accepted round or native UI/citation-click acceptance is claimed.
- Fresh automated gates pass: cargo fmt/check, default and QA-feature strict
  all-target Clippy, cargo test, typecheck/lint and Vitest247/247 in26 files;
  Node14/14, version0.8.0 and whitespace checks. Rust library95 pass/1 ignored;
  default integration98 passing rows include3 opt-in early returns, not live
  passes. All4 real local Whisper cases run36.27s. QA pipeline12/12 and
  review7/7 pass. Logs are `length-fixed-rust.log` and `final-frontend.log`.
  Node word-count comparison usesv22.14.0/ICU76.1; Rust uses ICU4X2.3.0.
- Default-feature Apple Silicon `Velo Extractive Preview.app` builds, with
  executable SHA256
  `00eb6438822ed74f66aba7eaf9220c63d5fdc6b642a35db542051455b81e285b`.
  It is not launched. Existing app memory is preserved. Hash comparison confirms
  no frozen runtime change during the final run and no change to the original
  transcript, production summary/settings or installed Info.plist.

Result: automated implementation/build checks pass, but the requested
three-format semantic/shape acceptance is **incomplete**. Stop API work here,
retain failed artifacts, and discuss neutral-heading policy and the150-word
example tradeoff before another implementation attempt. No commit, push, PR,
merge, tag, release, installed-app replacement or preview launch.

### Follow-up: acceptance example without a word ceiling

The user's reply approves removing the150-word limit from the custom example
only. The opted-in harness now requests a single paragraph without headings
or a list, keeping the important topics and complete conditions. Production
word-limit recognition/validation and the user's saved instructions are not
changed. The separate neutral-meeting-heading decision is still pending;
no new paid API call, accepted live round or preview launch follows from this
single approval. Earlier failures remain recorded above, not relabelled passes.
Fresh checks for this input-only amendment: cargo fmt/check, QA live-harness
compilation/listing (not execution), local Thai length tests2/2, explicit-limit
pipeline regression1/1 and whitespace checks pass. No full live or bundle
reverification is claimed for an input-only change while the heading decision
remains open.

## Neutral extractive summaries — 22 October 2026 (three-format round accepted)

- Both scoped changes are explicitly approved: remove only the acceptance
  example's150-word ceiling, and use neutral meeting headings. The existing
  layout validator now accepts only overview/topics for meeting excerpts;
  decisions/actions/open_questions fail locally before a provider's verdict
  could certify them. Key-point/custom heading policies and the explicit
  user-requested word-limit guard remain. Legacy saved Markdown is untouched.
- The provider-boundary regression is observed RED→GREEN: before the fix,
  “Maybe leave it for later?” is saved under Decisions despite its ambiguity;
  afterwards all three outcome categories either repair once to Topics or fail
  closed without reading a fabricated supported verdict or replacing the old
  cache. Existing canonical evidence/context, cancellation and length tests pass.
- Private evidence root: `/private/tmp/velo-neutral.WIICUp/`. Recovery archive
  SHA256`67a748c9359b45cc82c0bcc9e31ad361f458ac28e34bb407e3d6e8541f8a0bf1`.
  The live workspace's172-path manifest is `runtime-before.json`, aggregate
  `20fdc4bb31618832d123e7a445c9e10952dfb527d7ec813d4b14cde45d4ce0b6`.
  Frozen settings SHA256
  `12d824945597c682f52c6cffd519fbcd0da51fd0beae990a37db06188e7e31f6`;
  the custom override is bound by the frozen harness code. No API key/header is
  included in settings, request artifacts or reports; Rust Keychain reads it.
- Real DeepSeek run (session86243) exits0 in41.30s. All three formats save only
  into new private stores. Meeting12 logical calls/ceiling24; key points10/20;
  custom10/20:32 logical requests total, not a provider billing/retry count.
  Extraction uses6 packets each; review uses5/3/3 batches. No repair or further
  cloud round is needed on this policy. Earlier failed runs are not retroactive
  passes and do not count towards this round.
- Human/source comparison passes the agreed fixture criteria in all formats:
  person/group choice and later person Default remain quotations, not a delivery
  order; the completed-review removal question retains its uncertainty and
  consequence; read/unread backend retention does not become “unused now” or a
  fabricated unresolved issue. Shared-box and actual-reviewer passages remain
  in meeting/key points (and also custom here). No model-written factual prose,
  guessed name, owner, date or status enters the reader. Meeting has only Topics,
  key points only Key points, custom one paragraph/no heading/list. Unclear ASR
  remains visibly quoted; this is not corrected transcription or fluent prose.
- Independent artifact comparison in `audit.json` verifies every admitted
  visible point equals its whole original rows, and every attached clock belongs
  to those same rows with their original fractional starts. No review-only extra
  clocks are present. Source SHA256 remains
  `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`.
  There are19/12/12 visible occurrences. Thai word-like counts are644/441/523
  with Nodev22.14.0/ICU76.1; the523-word custom result is accepted because that
  example's ceiling was explicitly withdrawn, not because it satisfies150 words.
  All criterion flags are in `round-1/acceptance.json`. This is one complete
  accepted round, not three consecutive rounds or general-model certification.
- Native inspection discovers a presentation regression: a Topics-only summary
  appears empty because the old reader folds Topics by default. English/Thai
  first-render tests fail then pass after the minimal conditional-display fix.
  Legacy summaries with visible overview/outcomes still fold supplementary
  Topics; custom/key-point rendering is unchanged. Full frontend tests are now
  **249/249 in26 files**, not247. No summary text is changed or API called by
  this display fix.
- The post-live manifest `runtime-final.json` differs only in SummaryTab.vue
  among its frozen runtime paths. Every frozen Rust/config/native-test hash is
  unchanged; backend digest
  `84f6c8ab2c7e23f4fb0aad8439b51c1576957ceb05e4f6d0a9582d69938836f5`.
  Final runtime aggregate
  `469abad40437db2a341d796a659c43a8dfb220e3bc04dc06c6e25e11b1ab61ee`.
  No extra paid inference is used for this frontend-only rebuild.
- Final six-command gates and fmt check, QA all-target Clippy, Node14/14,
  version0.8.0 and whitespace pass (`rust-final.log`, `frontend-final.log`).
  Rust library95 pass/1 ignored; default integrations98 passing rows include
  the3 unchanged opt-in early-return rows, not exercised live passes. All4 real
  installed-Whisper cases run. QA pipeline12/12 and review7/7 pass. Default-feature
  Apple Silicon bundle builds (`bundle-final.log`); the existing STATIC_VCRUNTIME
  deprecation warning remains unrelated.
- Final native Preview is `com.velo.extractive-preview`, using a new app-data
  link to private `preview-data/` with copies of the tested transcript/meeting
  result and settings. It does not share caches or copy the production API key;
  the user must configure a key there before generating additional summaries.
  Only that temporary Preview is closed/reopened for its display rebuild; the
  existing Thai Font Preview is left running. The final summary is visible on
  initial file open, without expansion. A citation at02:13 seeks to02:13/03:44
  while paused; existing mounted tests verify exact fractional dispatch133.42.
  The Preview is left at the beginning, paused, player hidden, summary visible.
  No native new-generation test, new chat/microphone test or all-platform run is
  claimed. No commit, push, PR, merge, tag, release or installed-app replacement.

Final `handoff.json` confirms production transcript/summary/settings and installed
Info.plist hashes unchanged, along with the preceding running Preview executable
(`ad19dc746c2e9d7fcd0e5ce0dbcdf0c65ed8a7947c6b9b0c8453dddb851ee8ab`).
The final bundle executable hash is recorded there; its identifier/version are
com.velo.extractive-preview/0.8.0. Original app data is not linked into the preview.

## Direct AI summary replacement — 22 October 2026 (local, live acceptance incomplete)

- User approved direct model-written prose, context-based spelling correction,
  retained source times, three formats and additional DeepSeek calls. No push/release.
- Main runner is now `summary/pipeline.rs`; the preceding experiment is archived
  in `summary/evidence_pipeline.rs` and its integration fixtures explicitly call
  that archive. The app does not call extraction/layout/placement review.
- Direct tests exercise rewritten prose, one-call short input, source-clock
  resolution, leading-zero ID equivalence, one bounded citation repair, chunked
  notes/merge, oversized/invalid input and cancellation/cache preservation.
- Final `cargo fmt --check`, clippy all targets with warnings denied, and
  `cargo test` pass: 199 passed, 2 explicitly ignored; optional real-media tests
  retain their existing environment-dependent skips. Test log:
  `/private/tmp/velo-direct-final-rust-tests.log`. An earlier full run was killed
  when the same test executable was rebuilt during the run; the final run exits0.
- Frontend typecheck/lint, all259 tests in27 files and production frontend build
  pass. Direct progress/cancel routes are tested. This turn does not rebuild,
  restart, replace or claim a native Preview bundle using the new backend.
- Live runner `summary_direct_live` uses the original cached53-row transcript
  and saved DeepSeek key through Keychain. It writes only new isolated outputs.
  Raw transcript SHA256 remains
  `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`.
- Round1: meeting/custom generated, key points missing an overview citation;
  custom wrongly added a heading. Round2: all three rejected shortened source
  IDs such as S001. After numeric-equivalence normalization, round3 generates
  key points/custom; meeting still fails because the uncited grouping bullet
  `- งานที่ตกลงแล้ว` is treated as a factual block. No cache is saved for failure.
- Latest artifacts: `/private/tmp/velo-direct-summary-20261022-round3/`.
  Earlier rounds remain in sibling `round1` and `round2` directories. These
  attempts are NOT three passing rounds, nor one complete accepted round.
- Human comparison: custom is now one paragraph without a heading, Default is
  readable, and displayed labels resolve to actual source times. However both
  generated outputs still infer a negative ownership comparison from noisy
  source around00:59 (`ไม่ได้ดูจากสวยกันตัวเองนะ`); key points also adds standalone
  missing owner/deadline metadata. Valid clocks do not certify claim entailment.
  Further unchanged cloud reruns stopped; these limitations are reported to the
  user rather than relabelled as a semantic pass. Production data and running
  applications remain untouched.

## Direct summary heading/fidelity follow-up — 22 October 2026 (not semantically accepted)

- Fixed the reproduced grouping-label failure with an exact organizational-title
  allowlist. Known plain/bold/list labels can group cited text; arbitrary claims,
  unknown IDs and empty summaries are still rejected. Regression observed red
  before the change and green afterward.
- Added one bounded AI prose edit against the same original input per draft,
  including map notes and final merge. It is not Evidence Ledger: no extracted
  claim schema, placement verdicts, quotation composer or semantic retry loop.
  The draft is explicitly untrusted. Reviewed output is citation-checked before
  saving; review failure preserves the old cache. Existing citation repair stays
  bounded to one attempt before this edit. Full request size is checked, including
  the draft; an oversized review fails instead of truncating original sources.
- Seven direct regression tests cover original-source review, all three formats,
  numeric IDs without a repair call, bounded repair, long inputs and failure/cache
  behavior. The new review test was red before implementation. Mock responses
  verify wiring only; they do not demonstrate real-model semantic accuracy.
- DeepSeek live outputs are isolated in
  `/private/tmp/velo-direct-heading-20261022-round1/`, `round2/` and `round3/`.
  Round1 used stronger instructions only; rounds2–3 added bounded prose review.
  All three modes generate and pass local citation checks in each round. These
  are NOT three semantic passes. No additional cloud retries after round3.
- Round2 removed the guessed negative ownership comparison and separated click
  from completion, but meeting/key points invented a rollout sequence from the
  default setting. Round3 explicitly distinguishes default from implementation
  order and no longer shows that sequence in the inspected outputs.
- Round3 still fails acceptance: meeting invents an unresolved group-name
  question despite the subsequent explanation at01:55/02:01. Meeting/key points
  compress the group/department rights relationship into rights of the creator.
  Custom keeps the major conditions but attaches an overly broad citation cluster
  (including02:51–02:55) to read/unread storage. An existing clock is not proof
  that every attached excerpt supports the statement. No claim of all-mode
  semantic success or guaranteed accuracy is made.
- Frontend typecheck/lint and259 tests across27 files pass; log:
  `/private/tmp/velo-direct-heading-frontend.log`. Rust fmt/check and all-target
  Clippy with warnings denied pass. Final full Rust suite exits0:201 passed,
  2 explicitly ignored (existing optional environment-dependent early returns
  remain; these are not additional live acceptance passes). Final full Rust log:
  `/private/tmp/velo-direct-heading-final-rust.log`.
- The original transcript SHA256 remains
  `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`.
  Production settings/cache and running previews were not changed or restarted.
  No native bundle rebuilt, no commit, push, tag or release.

## Project library — 2026-10-22

- Implemented persistent display labels and project groups separately from the
  transient playback queue, with search, pinning, sorting, collapse preferences,
  confirmed writes and retryable errors. Media paths remain identity; labels do
  not rename files or change transcript/summary cache keys.
- Final gates exit0: typecheck, lint, Vitest (275 tests /31 files), cargo fmt
  --check, all-target Clippy with warnings denied, and cargo test (207 passed,
  2 ignored). Existing environment-gated live tests are not live acceptance runs.
  Logs: /private/tmp/velo-library-final-frontend.log and
  /private/tmp/velo-library-final-rust.log. Apple Silicon app bundle also built
  successfully (/private/tmp/velo-library-final-build.log).
- One final read-only review found three substantive issues, all fixed with
  RED-to-GREEN regression tests: queued preference lost updates, search changing
  persisted collapse, and conflict reload losing a rename draft.
- Native QA used com.velo.library-preview with app data isolated under
  /private/tmp/velo-library-qa.dwAzJf/data, not production settings or caches.
  Light/dark rendering, rename, missing-file retention and restart were exercised.
  Renamed label, pinned record and collapsed project survived restart without
  autoplay. Search revealed the collapsed match; clearing it restored collapse.
- Synthetic sample.wav retains SHA256
  7ec88a17fa0f6e986c17819a8ba692f1ea15c1cf9780d2ac6d78817d2d9cf5a6.
  Screenshot: /private/tmp/velo-library-qa.dwAzJf/library-final.png.
- Remaining QA: native narrow-window and 1,000-row responsiveness, full keyboard
  focus traversal and native corruption recovery have not been exercised.
  The 1,000-record derived view and corrupt storage are covered by automated tests.
  Minor deferred polish: editor cancellation does not restore trigger focus;
  name sorting uses host locale rather than the selected app locale.
- Existing dirty work is retained on the approved branch. No commit, push, tag,
  release, production migration or cloud summary API call was made for this task.

### Remaining library QA follow-up — 2026-10-22

- Tested the native Preview at640x600 with1,000 synthetic recordings across
  50 projects, including long Thai/English labels. Search for0999 returned the
  correct last record (1/1000); clearing returned1000/1000. Scrolling across
  groups, changing name sort and renaming record0999 with Enter succeeded.
  No visible freeze or layout overflow in these interactions. This is functional
  desktop QA, not a frame-time benchmark. Large accessibility snapshots were
  incomplete in the automation tool; screenshots and filtered snapshots verified
  the UI instead.
- Found and fixed keyboard defects: editor close lost focus; recording rename
  returned to a removed menu item; project actions ignored Escape; narrow library
  allowed Tab into the obscured workspace. Also handled Safari pointer clicks
  that do not focus buttons before opening editors. Six additional regression
  cases were observed failing before their fixes.
- Native final-build checks confirm forward/reverse Tab wraps inside the narrow
  library, Escape closes it and returns focus to its opener, Enter reopens with
  focus on Close, editor cancellation returns to its trigger (pointer/keyboard),
  project-menu Escape closes only that menu, and successful recording rename
  restores its persistent action control. Name editor Escape does not close the
  surrounding library.
- Final frontend gates exit0: typecheck, lint,281 tests/31 files. Full Rust
  fmt-check, Clippy --all-targets -- -D warnings and cargo test exit0 (207 passed,
  2 ignored; optional live suites remain environment-gated). Final native bundle
  build exit0; existing STATIC_VCRUNTIME deprecation warning remains.
- Screenshot: /private/tmp/velo-library-qa.dwAzJf/library-narrow-1000.png.
  Scale snapshot preserved at library-scale-tested.json in that directory;
  original two-record Preview fixture restored afterward. sample.wav hash remains
  unchanged. Production data, media filenames and caches were not modified.
- This completes the requested narrow/scale/keyboard follow-up. Previously noted
  host-locale name sorting remains unchanged. Native corrupt-copy recovery was
  not part of this follow-up; automated storage corruption coverage still passes.
  No push or release.

### Long-recording output/context recovery — 2026-10-23 (live acceptance incomplete)

- Continued on codex/long-summary-recovery, preserving the pre-existing local
  changes. User approved one DeepSeek output-limit restart with additional
  output room up to8192 tokens and real three-format tests, without publishing.
- RED: the real request-boundary test observed the old4096/2048 retry caps.
  GREEN: DeepSeek recovery gets8192 at32768 context, or4096 at16384; other
  dialects retain their limits. Recovery reuses identical supplied source,
  never interrupted text; repeated limits stop after two calls and preserve
  the cache. Cancellation before recovery starts no further request.
- The first authorised real follow-up at verified/ failed meeting mode with
  summary_context_limit after mapping7/7 and nonshrinking note reduction.
  Stopped the remaining run rather than claiming three-format success.
  Its artifacts are under /private/tmp/velo-long-summary-live.B6kTPn/verified/.
- A second RED regression reproduced unnecessary reduction even though the
  notes plus a bounded draft fit a reviewed merge. Review reservation now uses
  the bounded answer allowance rather than half a large context, retaining the
  half-window split for small contexts and the actual serialized-request guard.
  All12 long-recording tests pass, including synthetic1/2/3-hour timelines in
  all three formats, complete source coverage and original start/tail clocks.
- Fresh gates: cargo fmt and fmt --check, Clippy --all-targets -- -D warnings,
  full cargo test --no-fail-fast, typecheck, lint,319 Vitest tests/35 files,
  22 Node release checks, version consistency, frontend build and isolated
  aarch64 macOS app bundle all exit0. Hardware/provider-dependent tests remain
  opt-in; these generic-suite results are not real audio/AI quality evidence.
- Final-source pipeline SHA256:
  0408f6d7394743108f782f4984a344388e4d57468cb33f4c80f9100f9bf85630.
  Transcript: /Users/emozerorise/Downloads/voice_214468.aac, existing2322-segment
  cache through6252.52s, SHA256
  cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b.
  Main and Preview transcript files match byte-for-byte; neither was rewritten.
- After user Keychain approval, final/ failed meeting mode on the model's
  source-ID range [S001839-S001843], not on an output-limit completion. Stopped
  that old run when the user approved a narrow range-format correction.
- RED/GREEN regressions now expand a range only when every ordered source ID
  exists in the supplied set; missing/reversed/overflowing/malformed ranges
  remain invalid even next to a valid citation. Each expanded ID resolves its
  own original clock; the integration case checks00:37,00:43 and1:00:02 without
  a citation-repair call. Full Rust/Clippy and frontend gates pass afterward.
- Current validation SHA256:
  65809b25bbf55e88354ba0d00e4ed6d5f00b9d0ae97c05df4056541951cf8b17.
  The latest live runner uses deepseek-v4-pro at api.deepseek.com,32768 context,
  the existing Keychain key, original transcript and isolated range-verified/.
  Meeting generation completes (9300 UTF-8 bytes,316 references/304 unique
  source clocks,5 headings). Key points completes (15896 UTF-8 bytes,324
  references/319 unique clocks,2 headings). Every rendered clock belongs to the
  original transcript; no internal ID leaks. These are source-location/shape
  checks, not a guarantee that every model-written claim is entailed.
- Custom mode reads6/6 chunks, enters reducing0/2, then fails before the final
  answer with: Summary exceeds the context budget; increase the context window.
  Previous summary unchanged. The actual serialized request guard stops the
  job; no custom result/cache is saved. The three-format live test exits101
  after980.06s with Failed formats: [custom]. See range-verified/custom-error.txt
  and inspect.mjs in /private/tmp/velo-long-summary-live.B6kTPn/.
- The bounded output/context changes therefore do NOT complete the requested
  real-file acceptance. Stop further API patches/runs for a context-budget
  design review rather than weaken citation checks, discard source or search
  for a lucky pass. The request planner still estimates draft bytes, while the
  final guard measures the actual quoted/canonicalized draft; ID expansion can
  grow that draft beyond the estimate. Exact failing request sizes were not
  captured, so no claim is made about their numeric breakdown.
- All six code gates and the isolated aarch64 bundle pass; the pre-existing
  STATIC_VCRUNTIME bundle-build deprecation warning remains. The final bundle
  has a verified local ad-hoc resource signature, not Apple notarization. It
  was NOT opened because the user's three-format precondition is unmet. No
  source audio upload, production cache replacement, push or release.

### Actual-size follow-up — 2026-10-23 (real-file technical acceptance complete)

- User delegated the remaining fix choices and authorised additional API calls,
  retaining the three-format real-file gate before opening Preview. No further
  design-approval loop, push or release is part of this follow-up.
- RED/GREEN: oversized quoted review and citation-repair candidates previously
  hit the request guard. Expanded consecutive source IDs also inflated review
  requests. New tests preserve the complete supplied source and exercise both
  a valid independent review and rejection/cache preservation on invalid IDs.
- Generated notes/drafts compact adjacent consecutive canonical IDs on the
  wire only, without crossing claim/newline boundaries. Original transcript
  rows and final clock lookup are unchanged. Fully assembled follow-ups are
  measured for initial and recovery limits; if necessary only the generated
  candidate is excluded, and an independent corrected pass still receives all
  the same source and review/repair instructions. Byte estimates are not an
  exact provider tokenizer. Numeric-only diagnostics expose request sizes.
- Fresh local gates exit0: cargo fmt/fmt-check, Clippy all-targets with warnings
  denied, full cargo test --no-fail-fast (optional hardware/provider cases remain
  opt-in), typecheck, lint,319 Vitest tests/35 files,22 release checks, version
  consistency and git diff --check. All15 long-recording regressions pass,
  including the synthetic1/2/3-hour matrix across all three formats.
- Native aarch64 app bundle builds and passes local ad-hoc resource-signature
  verification. Existing STATIC_VCRUNTIME deprecation warning remains. The
  bundle is Velo Long Summary Recovery Preview.app with isolated Preview data
  and updater endpoints disabled; it is not opened before live acceptance.
- Frozen pipeline SHA256:
  ef4509759472c98465d61b0793cac67baa0bba9e30c4e487c0c7dc8879134c65.
  Validation SHA256:
  d111a5f2ad9463b7ed2da65c0ebfacf8bc9badd59a89c854eb0a0efa41adb187.
- The new real run writes only to
  /private/tmp/velo-actual-context.DDU0BQ/results, uses the unchanged2322-segment
  voice_214468.aac cache and32768-context deepseek-v4-pro settings, and runs
  custom first with stop-on-failure. Keychain access was approved by the user.
  Acceptance remains pending until the process exits and all three outputs are
  checked; local mock tests alone are not a real-file pass.

- That run generated custom without a context-limit failure, but independent
  inspection found9 paragraphs rather than the requested1, with excessive ID
  ranges. It was stopped (exit130) during the next mode instead of claiming
  three-format acceptance. Outputs remain under actual-context.DDU0BQ.
- RED: emitted review/repair requests ended with an untrusted candidate after
  the selected layout. GREEN: a compact selected-layout contract is repeated
  after that candidate and included in the budget check. Repeating the entire
  source reminder exceeded the8192-context regression; the compact contract
  avoids that duplication. All16 long-recording cases and the full Rust suite
  pass afterward; Clippy/fmt and frontend/release gates also exit0.
- The next frozen build has pipeline SHA256
  841455599ae5302263558c06eaa1db7ee7d9204696b44cf669c1eaa0d5cfd107,
  unchanged validation hash, and prompt SHA256
  140e0db4bedbba5c47b4a261788ab3e91a11a4a8261f386615860ea50edf8f86.
  New live results use /private/tmp/velo-final-layout.FtIVPL/results. No source
  rows or final references are removed to fit; source-ID ranges remain valid
  only when every included ID belongs to the supplied source.

- final-layout.FtIVPL exits101 in custom citation repair: the model emits
  [S000087-S000096], a range not fully present in the supplied notes. No custom
  summary is committed; later modes are skipped. Validation is deliberately
  NOT relaxed to accept unseen IDs from elsewhere in the recording.
- RED/GREEN: the repair request previously discarded the validator's actual
  error and gave only a generic instruction. It now includes bounded, JSON-
  quoted error-kind/reference feedback, without reattaching rejected prose,
  and asks for individual supplied IDs rather than potentially gapped ranges.
  Review keeps the trailing layout contract. All17 long regressions, the full
  Rust suite, fmt-check, warning-denying Clippy, typecheck, lint and319 frontend
  tests pass. The live harness now also stops on the custom paragraph mismatch.
- Latest frozen pipeline SHA256:
  675001db19514c16f0d641367b882b748bc6e7f033d6364124f186c95c4bbf0f;
  prompt SHA256:
  d570c3cbb210cf84f108849fc17be40bf9f94bc54500475057a2cff9ca6ed5cd;
  validation remains unchanged. Fresh actual results use
  /private/tmp/velo-final-reference.Bt1VbT/results. No extra repair/retry loop
  or provider-context setting change was added.

- final-reference.Bt1VbT exits101 during custom mapping2/6 after the bounded
  citation repair still emits unavailable[S000098]. No result/cache is saved,
  and the remaining formats do not run. Repeated prompt-only retries stop here.
- The follow-up makes source identity explicit rather than accepting/remapping
  guessed IDs: each request repeats its exact allowed ID set as losslessly
  compressed identity metadata, not evidence. The one citation repair restarts
  from complete supplied source without the invalid candidate. RED/GREEN
  emitted-request tests cover both removal of that anchor and the exact ID
  choices. No extraction/ledger/audit loop or additional retry is introduced.
- Current frozen pipeline SHA256:
  679d27684b371b3aedbe79455294a3983b46922a99394d2523bcae697d59b2cf.
  Prompt/validation hashes remain as above. Full Rust, Clippy/fmt, frontend
  gates and all17 long cases pass, and the matching isolated native bundle
  builds and verifies its local ad-hoc resource signature. Live outputs use
  /private/tmp/velo-reference-choices.DQRtwa/results.
- Custom completes on this build:34706 UTF-8 bytes,235 references/235 unique
  original source clocks,0 headings and1 paragraph as requested. The actual
  review observes a44376-byte draft (35842 quoted compact bytes) that would
  exceed the recovery budget with source; it excludes only the generated draft
  and completes from the same43801-byte supplied source. The review/recovery
  request has46945 user bytes and at most2646 system bytes against73728 allowed
  bytes. Neither source nor references are silently truncated.
- Manual opening/ending review also finds noisy ASR terms in the custom prose.
  Original-clock membership and successful generation do not certify every
  claim or fix the source transcription. The remaining two formats are still
  pending at this checkpoint; Preview has not been opened.

Final same-build acceptance:

| Format | UTF-8 bytes | References | Unique original clocks | Headings | Paragraph blocks |
| :--- | ---: | ---: | ---: | ---: | ---: |
| Meeting |9324|92|85|5|5|
| Key points |14857|166|159|2|28|
| Custom |34706|235|235|0|1|

- The real test exits0 (674.41s). Independent inspect.mjs exits0 with3/3
  formats,2322 source segments per result, the correct original media path,
  Thai language and deepseek-v4-pro. Every rendered clock belongs to an
  original source start, and no internal ID leaks. The custom test prompt's
  single paragraph/no headings/no lists is preserved. The complete answers,
  caches and numeric-only diagnostics remain under reference-choices.DQRtwa.
- These are successful generation, reference-location and shape checks, not
  a guarantee of semantic entailment. Manual opening/ending and full meeting
  reading still finds noisy ASR names/terms and potentially overconfident
  categories; those require source review and are not represented as fixed by
  this output/context work. No audio listening/transcription-quality pass is
  claimed. Optional hardware/provider suites remain opt-in.
- Opened the matching Velo Long Summary Recovery Preview.app only after3/3
  passed. Verified bundle identifier com.velo.preview, isolated updater-disabled
  config and local ad-hoc resource signature. Main and Preview transcript
  SHA256 still match the original cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b.
- Preview summary cache was empty; copied only the successful meeting cache
  using no-overwrite, then compared it byte-for-byte with the tested result.
  Native UI displays voice_214468.aac,01:44:12,meeting/deepseek-v4-pro and its
  summary. Confirmed the cached transcript opening matches the original and
  returned to summary view. Clicked00:08 and observed00:08/01:44:12; playback
  remains paused, transcript/chat closed and player open for the user.
- A clock with multiple original segments in the same floored second remains
  plain text under the existing conservative reader rule rather than guessing
  which segment to seek. Membership checks above do not certify every clock
  as an unambiguous clickable target. No validation rule was weakened.
- Final local gates: cargo fmt/fmt-check, cargo clippy --all-targets -- -D
  warnings, full cargo test --no-fail-fast, typecheck, lint,319 Vitest tests/35
  files,22 release checks, version consistency and git diff --check exit0.
  All17 long-recording cases pass; native aarch64 bundle build exits0 with only
  the pre-existing STATIC_VCRUNTIME deprecation warning. No production cache,
  transcript, audio or app was replaced. No commit, push or release.

## Detailed meeting settings and manual playback — 2026-10-23

Approved scope: Native sequential implementation, real voice_214468.aac
acceptance with additional DeepSeek calls, then matching Preview. No push or
release. Existing dirty long-summary recovery work is retained.

Local evidence:

- RED→GREEN: three real-mpv fixture tests for fresh load, playing-file
  replacement and resume/seek/explicit Play. Repeated with both the original
  AAC and the existing 3:44 MP4. Tests mute audio and use null video; they
  verify native pause and clock behavior, not a subjective listening test.
- Self-review found FILE_LOADED still emitted Playing unconditionally. All
  three fixture assertions failed against that event policy; the corrected
  production helper mirrors mpv pause and reports Playing only after Play.
- Settings/result tests cover missing/unknown detail→standard and unrelated
  settings preserved. Saved-result metadata, not next-run settings, controls
  detailed layout. Transactional save failure retains saved options and draft.
- Emitted-request tests cover detailed policy through draft/review/repair and
  the synthetic1/2/3-hour matrix, including source rows, original clocks,
  recovery budget, bounded reduction and cancellation/cache safety. Custom
  and key-point modes ignore stored meeting detail.
- UI tests cover three format radios, independent detail selection, advanced
  disclosure, example-overwrite confirmation, no inference on Save, focus
  wrap/Escape/restoration, standard/detailed reading and transport actions.
  A read-only guidance disclosure explicitly participates in modal Tab focus.
- Full frontend suite:330 tests/37 files. Rust suite:250 reported passes,
  including environment-gated test functions which skip work when not opted
  in; only the explicit fixture/cloud runs establish those real boundaries.
  fmt, Clippy -D warnings, typecheck, lint and22 release-policy checks pass.

First real acceptance (not a success):

- Isolated `/private/tmp/velo-detailed-meeting.hZKQUl/results`; the detailed
  case was intentionally first. It stopped with summary_context_limit after
  649.61s, saved no partial summary and prevented the other three billed cases.
- Numeric traces show six completed raw chunks followed by repeatedly similar
  reduction groups. Detailed note preservation reused raw-excerpt wording,
  so the model barely consolidated existing notes. A failing request-boundary
  assertion reproduces the missing reduction-specific instruction.
- Correction adds a detailed-only consolidation instruction and soft UTF-8
  writing target divided across groups. It asks to remove repetition, not
  material reasons/conditions. No hard cutting, source dropping, output-cap
  increase, new retries/passes or changes to other formats. Existing failure
  and cache protections remain if a provider cannot follow this guidance.
- Fresh full checks and a second isolated real acceptance are required below;
  the first failed run is retained as diagnostic evidence.

Synthetic browser layout acceptance (not a native zoom claim):

- A local-only fixture mounts the real SummaryPromptEditor with3330 characters
  of synthetic Thai draft text. Native command invocation is disabled; no
  credentials, production data, settings write or model requests are used.
- Light/dark at760x550 and at an equivalent380x275 CSS viewport scaled2x
  retain the fixed footer and scrollable body. At380x275 the footer occupies
  y197.25–248.25; document scrollWidth380 equals clientWidth380. No horizontal
  overflow; advanced instructions and the full draft remain reachable.
- Real browser ArrowLeft changes the native format radio from custom to key
  points. Tab from Save wraps to Close. Custom exposes the advanced editor;
  applying One paragraph asks for confirmation and Keep retains all3330
  characters. This supplements unit focus/draft tests, not a claim that the
  desktop app provides a200% zoom setting. Native Preview checks remain below.

Second real acceptance and exact-size regression:

- `results-consolidation` also ends with summary_context_limit (600.30s),
  before any complete result or other format. The soft consolidation target
  alone does not establish reliable reduction; no success is claimed for it.
- A new local fixture produces two complete24KB notes. The exact mandatory
  final/source-only-review requests fit the unchanged32768-token window,
  but the heuristic reserve for an optional draft forced reduction and failed.
  The regression reproduces this failure before the correction.
- Detailed mode now tests the actual mandatory draft, source-only review and
  worst-case bounded citation-repair messages before asking AI to reduce.
  These reuse the same source/allowed-ID/instruction builders as real calls,
  including JSON diagnostic expansion and both output ceilings. A generated
  candidate remains optional; complete note content is never cut to fit.
- The regression succeeds in exactly6 calls (two mapped pieces and the final
  reviewed merge), with no reduction calls. All20 long cases pass, including
  genuine over-budget reduction, non-progress/four-pass stops, source-ID
  rejection and previous-cache preservation. Other formats retain their
  existing reduction decision. Fresh full gates and frozen-build real
  acceptance still follow; this is not a provider/model-cap increase.

Exact-fit frozen-build live acceptance in progress:

- Pipeline SHA256 c1273f8a05c55dd8840b99f4a94a66fa493715ba4e9a1cdba7b495f2701bf1ac;
  prompt45b44351fc9cc244b2535c13d08de31a3e11beb1c581efeaa6df957b9d22085f;
  validationd111a5f2ad9463b7ed2da65c0ebfacf8bc9badd59a89c854eb0a0efa41adb187.
- `results-exact-fit/meeting_detailed.json` completes with18567 UTF-8 bytes,
  146 references/126 unique original clocks,5 headings and correct detailed
  metadata. It reaches final composition directly from51719 user bytes,
  without unnecessary reduction. Bounded output/citation recovery and the
  source-only review remain inside the existing73728-byte recovery budget.
- Manual reading compares the original00:00–00:55,15:00–16:50,30:40–32:15,
  42:50–44:30,52:20–53:25,1:26:00–1:27:58 and closing excerpts. Retained
  examples include requirement-based bug/request tracing, change-impact
  assessment and backup/restore/location conditions. This is materially more
  than simply repeating a short overview.
- Quality limitations remain visible in the unedited AI output: noisy opening
  terminology, some software/system names and the date28/final phrase are not
  independently established by ASR. Some conditional/template qualifications
  can still be compressed away. Clock membership does not prove entailment;
  this is not a claim that every name, date, category or sentence is correct.
  Users must review uncertain passages against audio before making decisions.
- The other three variants and native handoff checks are pending at this point.
  The user separately authorised backing up the existing Preview-only summary
  before displaying this tested detailed result, after all variants complete.

All-format sizing correction / acceptance reuse ruling:

- The exact-fit checkpoint's custom case was interrupted after it repeated
  unnecessary reductions. No custom result was saved or represented as passing.
  The same local48KB-note regression fails for standard mode before enabling
  the exact fit check in every format; all four variants pass afterward, in
  six requests each. All20 long tests and fresh full Rust/Clippy gates pass.
- Final pipeline SHA2568351e709463219476b34e6f048f478b386453d3c28af3a9e0b46464baa0866ac.
  Prompt and validator hashes above are unchanged. The only change to the
  accepted detailed branch is removing an always-true detailed-mode condition
  around the identical fit predicate (plus renaming that private helper).
  Its provider messages, limits and execution behavior are unchanged.
- Native execution ruling avoids billing this equivalent accepted detailed
  case again. Keep its original artifact/hash, and explicitly run only custom,
  standard meeting and key points under `results-remaining` on the final guard.
  This is not an identical-source-hash four-case run; the two checkpoints and
  independent artifact checks must remain visible in the handoff evidence.
- The summary entry button now matches the dialog name: Summary settings /
  ตั้งค่าการสรุป. Both locale SSR assertions failed before the copy change,
  then the complete330-test frontend suite, typecheck and lint passed.

Final checkpoint for this turn — NOT full acceptance:

- `deepseek_remaining_variants` stops after480.26s at custom with
  summary_output_limit. Exact fitting succeeds (no reduction loop): final
  draft receives45054 user bytes; its citation repair receives46933 bytes.
  The repair reaches both4096 and8192 output ceilings and remains incomplete.
  No complete custom result/cache is saved. Standard meeting and key points
  are intentionally not billed after this failure. Current-generation
  acceptance is therefore1/4 generated,1 failed,2 not run, not four passes.
- The accepted detailed result still has the ASR/interpretation qualifications
  noted above. Neither a successful terminal response nor source-clock
  membership certifies every semantic claim.
- Stop paid testing here; no further ceiling increase, content cutting,
  additional retry, provider change or speculative prompt rerun. Finishing
  the custom request while preserving its requested coverage requires a
  separately approved output strategy beyond the present fixed-ceiling plan.
- Local gates pass:251 reported Rust passes (opt-in functions may skip their
  work),330 frontend tests/37 files, typecheck, lint, fmt-check, strict Clippy,
 22 release-policy checks, version consistency and diff-check. Real native
  fixture checks explicitly pass3 AAC and3 MP4 cases.20 long-recording tests
  include1/2/3-hour all-format request/limit/cache contracts.
- The final updater-disabled aarch64 Preview builds and verifies its local
  ad-hoc resource signature. It has NOT been launched, and native-window
  acceptance is pending, because the approved real-file acceptance gate has
  not passed. The previously running Preview is left alone. The conditional
  backup/cache-replacement approval has NOT been exercised. No production or
  Preview source/cache/key was replaced. No commit, push or release.

## 2026-10-23 — Complete-part implementation: local gates pass, live gate fails

- Implemented the approved Native plan's Tasks1–5 without delegated agents,
  commits, push, release or automatic updater changes. Final review so far is
  author self-review, not independent review. Earlier failures above remain
  historical evidence and are not counted as acceptance for this build.
- Frozen summary/harness hash:
  5aeb3de27896cdc656254d35df57c2ca4b7878aac4f93167babe1dab4cf023c7.
  Source SHA256 remains
  cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b;
  all2322 transcript rows are unchanged. Main settings now select OpenAI, so
  the live test used unchanged Preview DeepSeek/deepseek-v4-pro/32768 settings
  and the previously authorized same-host Keychain key. No settings/key copy
  or OpenAI submission was made.
- Fresh local gates: fmt-check, strict all-target Clippy, cargo test285 reported
  passes/0 failures/4 explicitly ignored, typecheck, lint, Vitest338 tests across
 38 files,24 release-policy/artifact checks, version consistency and diff-check.
  The separately opted-in real AAC and MP4 manual-loading checks each ran3/3;
  file load/replacement/resume/seek stayed paused until explicit Play.
- The21 long-recording tests include a successful1/2/3h ×4variant matrix with
  original-row wire coverage, escaped Thai data, exact clocks and bounded
  requests. A separate denser3h fixture exceeds the unchanged128-call budget
  and fails safely while keeping the old cache. Duration alone is not a
  guarantee for arbitrary transcript density.
- Native runner tests cover whole-part assembly, remote conditions and later
  answers, source-scoped citations, empty results, cancellation, interrupted
  sweeps, second-sweep instability, and failed atomic rename retaining old
  cache bytes. Mock protocol success is not a model-quality certification.
- UI fixture uses actual SummaryTab/SummaryProgress in an isolated localhost
  640×400 iframe. Writing/checking stages have no fabricated percent; Cancel
  remains reachable, Tab focuses it and Enter calls summary_cancel. Localized
  old-result warning remains visible with a long paragraph;1:01:00 citation
  controls wrap and can be scrolled into view. The existing native app was
  never closed for this check. SSR/unit tests cover first-failure wording,
  changed totals, wrong-path events, late ready after cancel and reader state.
- The new acceptance verifier rejects missing/failed variants, wrong clocks/
  metadata/layout/identity/call bounds, leaked IDs and absent source-review
  notes. It correctly returns passed:false for the actual failed run; it is
  an artifact gate, not a semantic truth oracle.
- Actual run root: /private/tmp/velo-complete-parts.RxA551/acceptance.
  Custom wrote9 initial complete parts, completed the17-window first sweep,
  flagged6 for replacement and failed layout validation on a replacement
  after44 physical requests (summary_layout_unsupported). No completed
  summary artifact/cache was saved. Standard meeting, key points and detailed
  meeting were NOT attempted; source-semantic acceptance is NOT claimed.
  Rejected generated prose was not persisted, so the exact rejected structure
  (e.g. paragraph breaks versus headings) cannot be inferred from this record.
- Stop paid testing under the approved fail-fast rule. Do not silently relax
  the custom layout, retry the job, reuse older successes or claim four passes.
  Next debugging work needs a precise local rejection-shape diagnostic and a
  proven layout correction before another authorized four-case run.
- The updater-disabled aarch64 Velo Complete Parts Preview builds; local
  ad-hoc resource signing/deep strict verify passes (not Developer ID signing).
  It has NOT been launched because acceptance failed. The old Preview, its
  summary cache, production app/data and conditional backup approval remain
  untouched. Task6 and the previous integrated handoff stay incomplete.

## 2026-10-23 — Replacement layout fixed; consistency acceptance still pending

- The user authorized local repair and new paid acceptance in three formats,
  not a new release. Preserved the dirty checkout and every prior feature.
- Behavioral RED→GREEN reproduces (1) missing per-part ownership/layout after
  the untrusted draft, (2) plain cited paragraphs rejected before one-paragraph
  whitespace assembly and (3) a generated invalid shape misreported as an
  unsupported user prompt. Plain joining preserves words/IDs and bills no
  formatting retry; headings/lists/fences are not silently stripped. Exact
  same ending participates in preflight and actual review/repair requests.
- First frozen live build
  babadeaa800e3005b9bfeb1529ebf5fedb89ebf75a74b2f12c3a321239eda9d4:
  custom writes9 parts, completes17 first-sweep windows and4 replacements.
  The original layout failure does not recur. Second-sweep window3 fails
  routing validation at47 physical requests. No result is saved; other formats
  are not attempted. Root: /private/tmp/velo-layout-fix.eictNv/acceptance.
  The exact malformed routing response was intentionally not persisted, so
  do not assert whether this was JSON syntax, schema, a duplicate or an ID.
- Wire RED→GREEN then proves native JSON mode was absent from controls.
  Added response_format=json_object only to DeepSeek control requests, not
  prose/chat or other providers. Strict routing/schema/ID checks remain.
  Private rejection logs give serde category/position or unknown/duplicate/
  count, never raw source/replies/credentials. This is a syntax hardening,
  not proof of the prior rejected reply's exact cause or a truth oracle.
- Final frozen build
  b58aecedb314b0321a8ae9dba8bf2043c9a727d986070ddf433bc81098e3481e:
  fmt-check, strict all-target Clippy, Rust289 reported passes/0 failures/4
  explicitly ignored, typecheck/lint, frontend338 tests/38 files,25 release/
  artifact checks, version consistency and whitespace check pass. Opt-in
  tests are not represented as live passes. Real muted native AAC and MP4
  paused-loading/replacement/resume/seek checks each run3/3 on this code.
- Second isolated live root:
  /private/tmp/velo-layout-fix.eictNv/json-controls-acceptance.
  Original transcript hash remains
  cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b,
  2322 rows, Preview's DeepSeek/deepseek-v4-pro/32768 settings. The user
  directly unlocked each Keychain dialog; no key was printed/copied to logs.
  Explicit required scope is custom/meeting/key_points, not silently missing
  a detailed case. The verifier tests reject incomplete or smaller scopes.
- Custom writes9 parts, finishes17 first-sweep windows, revises9 parts without
  the old layout error and starts sweep2. Window5 returns a valid correction
  flag, yielding summary_review_unstable at59 calls. No malformed-JSON error
  occurs in this run. This flag does NOT prove which claim is wrong: the
  current routing object contains no rationale. It may reflect a genuine
  missed condition or a reviewer false positive. Do not claim semantic success.
- This follow-up uses106 real requests total (47+59) on two locally verified
  builds, each fail-fast. Meeting and key points are NOT run; accepted formats
  are0/3. No completed summary/cache is saved. The artifact verifier correctly
  reports passed:false. Stop further paid requests under this bounded gate.
- Updater-disabled Velo Layout Fix Preview builds and deep strict local
  resource verification passes after ad-hoc signing. It is NOT launched,
  because the required acceptance failed. No production/Preview settings,
  transcript, summary cache, library or credential was replaced. Existing app
  remains untouched. No commit, push, tag, release or automatic update.
- Next work requires targeted diagnostics that can compare the flagged
  candidate and original excerpts before changing the consistency policy;
  another blind prompt/cap/retry change would not establish fidelity. Native
  author self-review only; no claim of independent review or completed Task6.


## Direct-first summary simplification — 2026-10-23

User approval supersedes prior complete-part/sweep acceptance requirements.
The active runner sends fitting raw transcripts once; oversized input gets
whole-row excerpt summaries plus one combination. AI review, repair and
consistency/replacement calls are removed. Invalid or incomplete results do
not replace old cached summaries.

Local regression evidence: /private/tmp/velo-simple-red.log (4 expected
behavior failures), /private/tmp/velo-simple-green.log (5 passes). Extended
boundary tests characterize existing cache/provider protections under the new
pipeline; they are not claimed as additional RED/GREEN cycles. Full Rust run:
246 passes, 0 failures, 5 ignored; optional fixture functions may skip work.
Frontend: 338 passes, typecheck and lint pass. Clippy all-targets -D warnings,
fmt check, version check and release script tests pass. The new build is
isolated under com.velo.simple-summary-preview with no updater endpoints.

Original voice_214468 transcript SHA-256 remains
cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b,
2322 rows. Initial live command was rejected by automatic approval review
before execution because the transcript export to DeepSeek required explicit
payload/destination consent. No API call was made by that rejected command.
Live acceptance is pending that consent; do not infer success from local tests.


The user then explicitly authorized sending the original transcript to
https://api.deepseek.com with deepseek-v4-pro for all three paid tests.
The live process started and is waiting for macOS Keychain permission; no
credentials are printed. Separately, the opt-in original-transcript loopback
regression passed all three formats at four calls each (three excerpt calls
and one combination); every original quoted row was present. This verifies
request shaping only, not model content. Log:
/private/tmp/velo-simple-summary/original-local.log.

The isolated Velo Simple Summary Preview.app bundle builds successfully.
Its local ad-hoc resource signature was refreshed after bundling; deep strict
verification passes. This is not Developer ID signing or publication.


### Live follow-through after Keychain authorization

Initial run: custom completed in4 requests (three excerpts + one combination).
Meeting reached6 requests because one truncated excerpt was split, then failed
local final-reference validation at S000767 (not supplied in the notes).
Key-points was not called in that run. This is recorded in
/private/tmp/velo-simple-summary/acceptance/run.json; failed answer text was
not saved, so its exact context cannot be retrospectively assessed.

A narrow loopback test reproduced unsupported final IDs (reference-red.log).
The fix removes only unsupplied final-combination references and then runs the
same per-block validator. It does not rewrite prose, substitute another ID,
accept an entirely unreferenced block, or ask AI to fix anything. The full
Rust suite passes247 tests,0 failures,6 ignored; Clippy passes and the isolated
Preview rebuild succeeds. The new live run covers meeting/key_points only;
custom remains the prior successful artifact with its own original build hash.
This is explicitly a mixed-build acceptance record, not a claim of one frozen
build generating all three artifacts. User approved the paid destination and
unlocked Keychain for the rebuilt test executable.

Source spot checks: custom covers component discussion26:53–29:03, collection
register31:12–32:11, traceability52:21–53:25 and impact/cost authority1:26–1:28.
Component wording remains an interpretation of noisy speech; some late proper
terms are still garbled. No claim of full semantic fidelity follows from
reference membership or these spot checks. Do not bill extra review passes.


Second real meeting attempt:13 requests, with repeated output-limit source
splits into8 terminal excerpts; all excerpts completed, but their combined
notes exceeded final context before a final request could be sent. No semantic
review/repair was called. Key-points was again not reached by the old harness.
Log: /private/tmp/velo-simple-summary/reference-live.log.

Two loopback tests reproduced context/output-limit assembly failure, then
passed with lossless local built-in assembly. When one final answer cannot
fit, completed meeting/key-point excerpts are presented in chronological
sections without another model call, discarded content or recursive reduction.
Custom contracts are not silently changed. Excerpt wording now asks for a
2000-token target below its4096 cap; all original rows are still supplied.
Full Rust suite:249 passes,0 failures,6 ignored. Preview rebuilt and local
ad-hoc deep strict verification passed. Live harness now runs each selected
remaining variant once and records errors, rather than starving key-points
behind a meeting failure. Final attempt is waiting for Keychain authorization.


Final acceptance after Keychain approval (2026-10-23): meeting and key_points
completed, each with3 excerpt calls +1 combination call, no splits, review,
repair or local-assembly fallback in this run. Generation time excluding
Keychain: meeting125s, key_points121s. Custom retained its earlier4-call88s
artifact. All three pass saved/cache equality,2322 source-segment identity,
format checks and timestamp membership against the original source. Report:
/private/tmp/velo-simple-summary/validation.json. Final live test:1 passed,
0 failed. Final Clippy passed after the harness change. No further paid runs.
Total physical paid requests across these attempts:31 (12 in successful
artifacts,19 in the two failed meeting attempts). This is not a cost estimate.

Manual source spot-checks include the opening project-plan exception, component
discussion27:30–29:03, traceability52:21–53:25, impact assessment1:26–1:28
and go-live deadline1:42–1:43. Main themes are present, but semantic acceptance
is qualified: the meeting output calls the opening record “version control
record” while the transcript is garbled and the key-points output interprets
it as Validation Record. Component definitions also blur source/component
versus built software. Both outputs carry garbled proper terms from ASR;
key-points is lengthy because the saved user instruction requests detail.
Reference membership verifies navigable timestamps, not factual entailment.
No claim of sentence-level accuracy or exhaustive semantic coverage is made,
and no model review loop was added to disguise these limitations.

The isolated Velo Simple Summary Preview.app bundle is available under
src-tauri/target/aarch64-apple-darwin/release/bundle/macos/. It was built and
ad-hoc verified locally, not installed or launched. No push, tag or release.


2026-10-23 Preview UI follow-up: the transcription-device status was the
only unpainted strip between opaque reader/header surfaces in a transparent
webview. Added its own opaque surface background. New settings default to Thai
and hide summary timestamps (both Rust and frontend); saved explicit choices
are preserved. Hidden summary rendering strips timestamp tokens before parsing
so unsupported/duplicate source times do not remain as plain text. Cached
markdown is unchanged and showing timestamps still enables valid navigation.
The navigation test reproduced both initially visible citations and leaked
plain-text timestamps before the fixes. English-specific UI fixtures now set
their locale explicitly, and citation tests explicitly enable timestamps.
The user's running Preview already has Thai/hidden timestamps saved. Its chat
is memory-only; no forced restart was performed to avoid losing that session.

UI follow-up validation:338 frontend tests and249 Rust tests passed (6 live/
optional tests ignored); typecheck, lint, cargo fmt and diff checks passed.
The first sandboxed Rust run failed4 evidence tests at loopback bind with
PermissionDenied; the complete authorized local-mock rerun passed. Preview
bundle rebuilt successfully with the final hidden-text fix. No new paid API
calls, push or release. Native visual confirmation of the new GPU background
remains for the next launch; the existing running process was left intact.


Pre-push review, 2026-10-23: user confirmed the rebuilt Preview works normally.
Reviewed the combined working-tree changes in summary routing/generation,
bounded partitioning, reference rendering, transport completion, settings
compatibility, summary options and manual playback. No new blocking defect
identified in this review. Fresh checks:338 frontend tests,249 Rust tests
(6 optional ignored),25 release-tooling tests; typecheck, lint, cargo fmt,
Clippy --all-targets -D warnings and diff whitespace checks all passed.
The3 opt-in manual-loading tests additionally ran against the original local
voice_214468.aac with muted libmpv and passed (open, replace, resume/seek/play).
No paid API calls were made for this preflight.

All version sources still agree on0.9.0, already released; changes remain in
the unreleased changelog. Before a new release, choose a new version and use
the version script after the feature merge, then merge the release commit
before tagging. Windows and Intel macOS packaging were not executed locally;
the release workflow must validate those artifacts. The working tree has45
modified tracked files and18 new files, including required source/test
modules; committing only tracked changes would omit required modules.
No commit, push, tag or release performed during this preflight.
