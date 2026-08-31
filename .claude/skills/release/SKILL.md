---
name: release
description: Cut a Velo release — changelog, version bump, tag, and the GitHub build. Use when asked to release, ship, publish, tag a version, or when a merged feature is ready to go out. Also covers recovering a tag that was pushed at the wrong commit.
---

# Releasing Velo

A tag is what starts a release, and the workflow reads its notes from the
`CHANGELOG.md` section matching that tag. Both facts decide the order below:
**nothing is tagged until the changelog and the version bump are on `main`.**

## The order

### 1. Changelog first, in Thai

Add a `## [ยังไม่เผยแพร่]` heading above the newest version and write the
entry under it. Thai, in the voice of the existing entries: what changed, why
it matters, and the measurements behind it. `scripts/version.mjs` refuses to
run without that heading, and the release workflow fails without a section
matching the tag.

Sections in use: `### เพิ่มเข้ามา`, `### แก้ไข`, `### เปลี่ยนแปลง`,
`### เครื่องมือและงานภายใน`, `### ข้อจำกัดที่ทราบ`.

### 2. Merge the feature work

```bash
gh pr merge <pr> --merge
```

### 3. Bump every version source at once, on merged `main`

```bash
git checkout main && git pull && npm run release <version>
```

One command rewrites `package.json`, `src-tauri/tauri.conf.json`,
`src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` together, dates the
unreleased heading, and adds its link. Never edit them by hand: Tauri stamps
the app's version from `tauri.conf.json`, and 0.1.1 once shipped as a file
named 0.1.1 holding an app reporting 0.1.0.

### 4. Release commit, PR, merge

```bash
git checkout -b chore/release-<version> && git commit -qam "chore: release <version>" && git push -u origin chore/release-<version>
```

Then `gh pr create --title "Release <version>"` and merge once CI is green.

### 5. Tag last, and only after that merge

```bash
git checkout main && git pull && git tag -a v<version> -m "Velo <version>" && git push origin v<version>
```

Pushing the tag runs `.github/workflows/release.yml`: it creates the release
with notes from the `[<version>]` changelog section plus
`.github/release-footer.md`, then attaches `.dmg` installers for Apple Silicon
and Intel. Around 11 minutes. Watch it with:

```bash
gh run watch $(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')
```

## Before tagging

- `cargo fmt` — CI runs `cargo fmt --check` and it is the easiest way to fail.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, `npm run lint`,
  `npm run typecheck`, `npx vitest run`.
- **When native dependencies changed, build the bundle locally.** CI compiles
  with host defaults while `tauri build` applies the bundle config, so a
  release-only failure passes every other check — that is how a 10.13
  deployment target broke 0.3.0:

```bash
npm run tauri build -- --bundles app --target aarch64-apple-darwin
```

## When a tag lands on the wrong commit

Tagging before the merge points the tag at old code, and the run dies in
seconds with `No [<version>] section in CHANGELOG.md`. Nothing is published,
so retagging is free — but check `gh release list` first, and never delete a
tag that already has a release with assets attached.

```bash
git push origin :refs/tags/v<version>
```

```bash
git checkout main && git pull && git tag -d v<version> && git tag -a v<version> -m "Velo <version>" && git push origin v<version>
```

## Worth knowing

- Builds are **not signed** — there is no Apple Developer account, and the
  release footer already explains the first-launch warning to users. Do not
  propose signing as a fix.
- **macOS only.** Windows compiles in CI but ships nothing: libmpv needs a dev
  archive whose name carries a date and hash, an import library generated from
  `mpv.def`, and `libmpv-2.dll` bundled alongside. The header comment in
  `release.yml` has the details.
- `workflow_dispatch` on the Release workflow rebuilds an existing tag's
  assets without moving anything.
