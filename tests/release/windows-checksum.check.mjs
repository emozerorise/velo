import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const script = resolve('scripts/prepare-windows-asset.mjs');
function fixture(t) {
  const dir = mkdtempSync(join(tmpdir(), 'velo-checksum-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}
function run(path) {
  return spawnSync(process.execPath, [script, path], { encoding: 'utf8' });
}
test('normalizes the installer before hashing so downloaded checksum resolves its filename', t => {
  const dir = fixture(t);
  const source = join(dir, 'Velo Preview_0.9.0-beta.1_x64-setup.exe');
  const target = join(dir, 'Velo.Preview_0.9.0-beta.1_x64-setup.exe');
  writeFileSync(source, 'abc');
  const result = run(source);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), target);
  assert.equal(existsSync(source), false);
  assert.equal(readFileSync(target, 'utf8'), 'abc');
  assert.equal(readFileSync(`${target}.sha256`, 'utf8'),
    'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  Velo.Preview_0.9.0-beta.1_x64-setup.exe\n');
  assert.equal(run(target).status, 0);
});
test('refuses a name collision without overwriting either installer', t => {
  const dir = fixture(t);
  const source = join(dir, 'Velo Preview-setup.exe');
  const target = join(dir, 'Velo.Preview-setup.exe');
  writeFileSync(source, 'abc');
  writeFileSync(target, 'existing');
  assert.notEqual(run(source).status, 0);
  assert.equal(readFileSync(source, 'utf8'), 'abc');
  assert.equal(readFileSync(target, 'utf8'), 'existing');
  assert.equal(existsSync(`${target}.sha256`), false);
});
test('rejects names that would require additional GitHub normalization', t => {
  const dir = fixture(t);
  const source = join(dir, 'Velo+Preview-setup.exe');
  writeFileSync(source, 'abc');
  assert.notEqual(run(source).status, 0);
  assert.equal(readFileSync(source, 'utf8'), 'abc');
});
