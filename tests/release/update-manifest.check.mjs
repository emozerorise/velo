import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

function fixture(t) {
  const dir = mkdtempSync(join(tmpdir(), 'velo-manifest-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  for (const arch of ['aarch64', 'x64']) {
    writeFileSync(join(dir, `Velo_0.6.0_${arch}.app.tar.gz`), 'fixture archive');
    writeFileSync(join(dir, `Velo_0.6.0_${arch}.app.tar.gz.sig`), `signature-${arch}\n`);
  }
  return dir;
}
function run(dir, version = '0.6.0') {
  return spawnSync(process.execPath, ['scripts/update-manifest.mjs', version, dir, join(dir, 'latest.json')], { encoding: 'utf8' });
}
test('publishes only the two supported macOS architectures with embedded signatures', (t) => {
  const dir = fixture(t);
  assert.equal(run(dir).status, 0);
  const manifest = JSON.parse(readFileSync(join(dir, 'latest.json'), 'utf8'));
  assert.equal(manifest.version, '0.6.0');
  assert.ok(!Number.isNaN(Date.parse(manifest.pub_date)));
  assert.equal(typeof manifest.notes, 'string');
  assert.deepEqual(Object.keys(manifest.platforms).sort(), ['darwin-aarch64', 'darwin-x86_64']);
  for (const [platform, arch] of [['darwin-aarch64', 'aarch64'], ['darwin-x86_64', 'x64']]) {
    assert.equal(manifest.platforms[platform].url, `https://github.com/emozerorise/velo/releases/download/v0.6.0/Velo_0.6.0_${arch}.app.tar.gz`);
    assert.equal(manifest.platforms[platform].signature, `signature-${arch}`);
  }
});
test('refuses to publish when either architecture or its signature is absent or empty', (t) => {
  for (const arch of ['aarch64', 'x64']) {
    for (const suffix of ['', '.sig']) {
      for (const missing of [true, false]) {
        const dir = fixture(t);
        const path = join(dir, `Velo_0.6.0_${arch}.app.tar.gz${suffix}`);
        if (missing) rmSync(path); else writeFileSync(path, suffix ? ' \n' : '');
        const result = run(dir);
        assert.notEqual(result.status, 0, `${arch}${suffix}, missing=${missing}`);
        assert.match(result.stderr, /missing|empty|ENOENT/i);
      }
    }
  }
});
test('rejects unsafe versions before forming filenames or URLs', (t) => {
  const dir = fixture(t);
  assert.notEqual(run(dir, '../bad').status, 0);
});
