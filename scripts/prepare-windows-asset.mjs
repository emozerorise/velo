import { createHash } from 'node:crypto';
import { copyFileSync, constants, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';

try {
  if (process.argv.length !== 3) throw new Error('Expected one NSIS installer path');
  const source = resolve(process.argv[2]);
  // GitHub normalizes spaces on upload, but cannot rewrite checksum contents.
  const name = basename(source).replaceAll(' ', '.');
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*-setup\.exe$/.test(name)) {
    throw new Error('Unsupported release asset name');
  }
  const target = join(dirname(source), name);
  if (source !== target) {
    // Exclusive copy avoids silently replacing a different existing installer.
    copyFileSync(source, target, constants.COPYFILE_EXCL);
    unlinkSync(source);
  }
  const hash = createHash('sha256').update(readFileSync(target)).digest('hex');
  writeFileSync(`${target}.sha256`, `${hash}  ${name}\n`, 'ascii');
  console.log(target);
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
