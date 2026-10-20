# Velo transcript-first workspace QA

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
