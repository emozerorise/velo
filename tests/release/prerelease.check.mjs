import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

function fixture(t) {
  const dir = mkdtempSync(join(tmpdir(), 'velo-version-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  mkdirSync(join(dir, 'src-tauri'));
  for (const path of ['package.json', 'src-tauri/tauri.conf.json']) writeFileSync(join(dir, path), '{"version":"0.8.0"}\n');
  writeFileSync(join(dir, 'src-tauri/Cargo.toml'), '[package]\nname = "velo"\nversion = "0.8.0"\n');
  writeFileSync(join(dir, 'src-tauri/Cargo.lock'), '[[package]]\nname = "velo"\nversion = "0.8.0"\n');
  writeFileSync(join(dir, 'CHANGELOG.md'), '## [ยังไม่เผยแพร่]\n\nทดสอบ\n\n[0.8.0]: old\n');
  return dir;
}
function run(dir, version) {
  return spawnSync(process.execPath, [resolve('scripts/version.mjs'), version], { cwd: dir, encoding: 'utf8' });
}
test('beta bump updates every source, keeps readable notes and passes consistency check', t => {
  const dir = fixture(t);
  assert.equal(run(dir, '0.9.0-beta.1').status, 0);
  assert.equal(JSON.parse(readFileSync(join(dir, 'package.json'))).version, '0.9.0-beta.1');
  assert.equal(spawnSync(process.execPath, [resolve('scripts/version.mjs')], { cwd: dir }).status, 0);
  assert.match(readFileSync(join(dir, 'CHANGELOG.md'), 'utf8'), /\[0\.9\.0-beta\.1\]: https:\/\/github.com\/emozerorise\/velo\/releases\/tag\/v0.9.0-beta.1/);
});
test('rejects invalid versions and mismatched sources before any writes', t => {
  for (const value of ['../bad', '01.9.0', '0.9.0-beta.01', '0.9.0;echo bad']) {
    const dir = fixture(t);
    assert.notEqual(run(dir, value).status, 0);
    assert.equal(JSON.parse(readFileSync(join(dir, 'package.json'))).version, '0.8.0');
  }
  const dir = fixture(t);
  writeFileSync(join(dir, 'package.json'), '{"version":"0.7.0"}');
  assert.notEqual(run(dir, '0.9.0-beta.1').status, 0);
  assert.equal(JSON.parse(readFileSync(join(dir, 'package.json'))).version, '0.7.0');
});
test('release routing isolates beta from stable and rejects mismatches', async () => {
  const { releasePolicy } = await import('../../scripts/release-policy.mjs');
  assert.deepEqual(releasePolicy('v0.9.0-beta.1', '0.9.0-beta.1'), { preview: true, latest: false, updater: false });
  assert.deepEqual(releasePolicy('v0.9.0', '0.9.0'), { preview: false, latest: true, updater: true });
  assert.throws(() => releasePolicy('v0.9.0', '0.9.0-beta.1'));
  assert.throws(() => releasePolicy('v01.9.0', '01.9.0'));
});
test('Preview configuration replaces endpoints and separates app identity', () => {
  const base = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  const preview = JSON.parse(readFileSync('src-tauri/tauri.preview.conf.json', 'utf8'));
  assert.equal(preview.identifier, 'com.velo.preview');
  assert.notEqual(preview.identifier, base.identifier);
  assert.equal(preview.productName, 'Velo Preview');
  assert.deepEqual(preview.plugins.updater.endpoints, []);
  assert.equal(preview.bundle.createUpdaterArtifacts, false);
  assert.equal(base.plugins.updater.endpoints.length, 1);
});
test('stable Windows keeps production identity but disables native updater endpoints and artifacts', () => {
  const base = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  const windows = JSON.parse(readFileSync('src-tauri/tauri.windows.conf.json', 'utf8'));
  assert.equal(windows.identifier ?? base.identifier, 'com.velo.player');
  assert.equal(windows.productName ?? base.productName, 'Velo');
  assert.deepEqual(windows.plugins?.updater?.endpoints, []);
  assert.equal(windows.bundle.createUpdaterArtifacts, false);
  assert.equal(base.plugins.updater.endpoints.length, 1);
});
