# Working on Velo

Velo is a desktop video player — Tauri v2 + Rust behind a Vue 3 overlay, with
`libmpv` rendering to a native GPU surface *underneath* a transparent webview.
It also transcribes meetings offline (whisper.cpp) and summarises them with a
chat model the user runs themselves.

Read `PLAN.md` before changing anything structural: it carries the
architecture and the reasoning, including the measurements behind decisions
that otherwise look arbitrary.

## Setup

`CONTRIBUTING.md` has the full list. The two that catch people out:

- **libmpv + pkg-config** — `brew install mpv pkg-config` on macOS.
- **CMake** — `brew install cmake`. whisper.cpp is compiled into the binary,
  so without it the Rust build fails outright.

## Commands

```bash
npm run tauri dev        # run the app
npm run typecheck        # vue-tsc
npm run lint             # eslint
npx vitest run           # frontend tests
```

```bash
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo fmt
```

**CI runs `cargo fmt --check`** and it is the easiest way to fail a green-
looking branch. Run `cargo fmt` before pushing.

## Conventions

**Language is split deliberately, and the two halves are not translations of
each other:**

| Thai | English |
| :--- | :--- |
| `CHANGELOG.md`, GitHub release notes, `.github/release-footer.md` | commit messages, PR titles and bodies, code comments, `README.md`, `CONTRIBUTING.md`, this file |

The app's own UI is bilingual: strings live in `src/locales/`, `en.ts` is the
source of truth, and `th.ts` is typed against it — a missing translation is a
compile error, not a silent fallback.

Comments explain *why*, not what. Match the density and voice of the file you
are editing.

## Where things are

| Area | Path |
| :--- | :--- |
| Player core, mpv FFI, rendering | `src-tauri/src/player/`, `src-tauri/src/platform/` |
| Transcription (audio extract + whisper) | `src-tauri/src/transcript/` |
| Summarisation (chunking, prompts, transports) | `src-tauri/src/summary/` |
| IPC commands | `src-tauri/src/commands/` |
| Stores, services, components | `src/stores/`, `src/services/`, `src/components/` |

Rust modules stay independent of each other; where two features must meet,
they meet in `commands/`. The summariser is handed a transcript rather than
reaching for one.

## Gotchas worth knowing before you spend a day on them

- **CI cannot see release-build failures.** `cargo build/clippy/test` use host
  defaults, while `tauri build` applies the bundle config. A 10.13 deployment
  target once broke a release after every check passed. When native
  dependencies change, build the bundle locally before tagging:
  `npm run tauri build -- --bundles app --target aarch64-apple-darwin`.
- **Transcription tests skip themselves** unless pointed at real inputs.
  `VELO_WHISPER_MODEL=/path/to/model.bin` avoids re-downloading 547 MB, and
  `VELO_TEST_MEDIA` supplies a clip.
- **Summary tests are two-tier.** The mock-server tests in
  `src-tauri/tests/summary_pipeline.rs` always run. The live one skips unless
  `VELO_SUMMARY_LIVE=1`, needs Ollama running locally, and takes
  `VELO_TEST_TRANSCRIPT=<cached transcript json>` to run a real meeting
  through it. Do not make the live test mandatory — most environments have no
  model server.
- **Releases are not signed.** There is no Apple Developer account; the
  release notes tell users how to get past the first-launch warning. Do not
  propose signing as a fix.
- **Windows compiles in CI but ships nothing.** libmpv there needs a dev
  archive whose name carries a date and hash, an import library generated from
  `mpv.def`, and `libmpv-2.dll` bundled alongside. The header comment in
  `.github/workflows/release.yml` has the details.
- **The summariser talks to Ollama over its native `/api/chat`,** not the
  OpenAI-compatible route, because that is the only way to switch qwen3's
  reasoning off — measured at 22 generated tokens against 409 for the same
  summary. `PLAN.md` §31.4 lists the rest of the traps in that path, including
  why chunk budgets are counted in bytes and why the citation rule is repeated
  after the transcript rather than only before it.

## Releasing

A tag starts the release and the workflow reads its notes from the changelog
section matching that tag, so nothing is tagged until the changelog entry and
the version bump are merged to `main`. `npm run release <version>` sets every
version source at once. The full order, and how to recover a tag pushed at the
wrong commit, is in `.claude/skills/release/SKILL.md` — plain Markdown, useful
to any agent or person.
