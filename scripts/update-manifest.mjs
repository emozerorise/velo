#!/usr/bin/env node
// Publish a manifest only after both repaired bundles and their signatures exist.
import { readFileSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [, , version, directory, output, notesFile] = process.argv;
if (!/^\d+\.\d+\.\d+$/.test(version ?? '') || !directory || !output) {
  throw new Error('Usage: update-manifest.mjs <version> <asset-directory> <output> [notes-file]');
}
const platforms = {};
for (const [platform, arch] of [['darwin-aarch64', 'aarch64'], ['darwin-x86_64', 'x64']]) {
  const archive = `Velo_${version}_${arch}.app.tar.gz`;
  const path = join(directory, archive);
  if (!statSync(path).isFile() || statSync(path).size === 0) throw new Error(`Empty archive: ${archive}`);
  const signature = readFileSync(`${path}.sig`, 'utf8').trim();
  if (!signature) throw new Error(`Empty signature: ${archive}`);
  platforms[platform] = {
    url: `https://github.com/emozerorise/velo/releases/download/v${version}/${archive}`,
    signature,
  };
}
writeFileSync(output, JSON.stringify({
  version,
  notes: notesFile ? readFileSync(notesFile, 'utf8').trim() : '',
  pub_date: new Date().toISOString(),
  platforms,
}, null, 2) + '\n');
