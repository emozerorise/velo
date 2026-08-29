#!/usr/bin/env node
// Keeps every place that carries the app version in step, and rolls the
// changelog when releasing.
//
//   node scripts/version.mjs            check that all sources agree
//   node scripts/version.mjs 0.1.2      set them all, and date the changelog
//
// They drifted once already: 0.1.1 was bumped in package.json alone, which
// would have produced a file named 0.1.1 holding an app still reporting
// 0.1.0, since Tauri stamps the version from tauri.conf.json.

import { readFileSync, writeFileSync } from 'node:fs';

const UNRELEASED = '## [ยังไม่เผยแพร่]';

/** Each source, with how to read and rewrite the version it holds. */
const sources = [
  {
    file: 'package.json',
    read: (t) => JSON.parse(t).version,
    write: (t, v) => t.replace(/("version":\s*)"[^"]+"/, `$1"${v}"`),
  },
  {
    file: 'src-tauri/tauri.conf.json',
    read: (t) => JSON.parse(t).version,
    write: (t, v) => t.replace(/("version":\s*)"[^"]+"/, `$1"${v}"`),
  },
  {
    // Anchored at line start so dependency versions, which are inline in
    // their tables, cannot match.
    file: 'src-tauri/Cargo.toml',
    read: (t) => t.match(/^version = "([^"]+)"/m)?.[1],
    write: (t, v) => t.replace(/^version = "[^"]+"/m, `version = "${v}"`),
  },
  {
    file: 'src-tauri/Cargo.lock',
    read: (t) => t.match(/name = "velo"\nversion = "([^"]+)"/)?.[1],
    write: (t, v) => t.replace(/(name = "velo"\nversion = )"[^"]+"/, `$1"${v}"`),
  },
];

const read = (file) => readFileSync(file, 'utf8');

function check() {
  const found = sources.map((s) => ({ file: s.file, version: s.read(read(s.file)) }));
  const missing = found.filter((f) => !f.version);
  if (missing.length) {
    console.error('Could not find a version in:');
    for (const m of missing) console.error(`  ${m.file}`);
    process.exit(1);
  }

  const versions = [...new Set(found.map((f) => f.version))];
  if (versions.length > 1) {
    console.error('Version mismatch:');
    for (const f of found) console.error(`  ${f.version.padEnd(10)} ${f.file}`);
    console.error('\nRun `npm run release <version>` to set them together.');
    process.exit(1);
  }

  console.log(`All sources agree on ${versions[0]}`);
  return versions[0];
}

function bump(next) {
  if (!/^\d+\.\d+\.\d+$/.test(next)) {
    console.error(`Not a version: ${next} (expected e.g. 0.1.2)`);
    process.exit(1);
  }

  // Work out every edit before touching disk: a failure part way through
  // would otherwise leave some files bumped and others not.
  const changelog = read('CHANGELOG.md');
  if (!changelog.includes(UNRELEASED)) {
    console.error(
      `CHANGELOG.md has no "${UNRELEASED}" heading to release.\n` +
        'Add the entries for this version first -- the release workflow reads\n' +
        'its notes from the section matching the tag and stops without one.'
    );
    process.exit(1);
  }

  const edits = sources.map((source) => {
    const before = read(source.file);
    const after = source.write(before, next);
    if (before === after) {
      console.error(`Nothing to replace in ${source.file}`);
      process.exit(1);
    }
    return { file: source.file, after };
  });

  for (const edit of edits) {
    writeFileSync(edit.file, edit.after);
    console.log(`  ${edit.file} -> ${next}`);
  }

  // Local date, not UTC: an evening release in UTC+7 would otherwise be
  // stamped with yesterday's date.
  const today = new Date().toLocaleDateString('sv-SE');
  const url = `https://github.com/emozerorise/velo/releases/tag/v${next}`;
  writeFileSync(
    'CHANGELOG.md',
    changelog
      .replace(UNRELEASED, `## [${next}] - ${today}`)
      .replace(/^\[(\d+\.\d+\.\d+)\]: /m, `[${next}]: ${url}\n[$1]: `)
  );
  console.log(`  CHANGELOG.md -> [${next}] - ${today}`);

  console.log(`\nNext: commit, merge, then\n  git tag -a v${next} -m "Velo ${next}"\n  git push origin v${next}`);
}

const [, , requested] = process.argv;
if (requested) {
  bump(requested);
} else {
  check();
}
