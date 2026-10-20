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
