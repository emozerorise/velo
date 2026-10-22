import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export const versionPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?$/;
export function releasePolicy(tag, version) {
  if (!versionPattern.test(version) || tag !== `v${version}`) throw new Error('Tag/version mismatch or invalid version');
  const preview = version.includes('-');
  return { preview, latest: !preview, updater: !preview };
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const version = JSON.parse(readFileSync('package.json', 'utf8')).version;
  const policy = releasePolicy(process.argv[2], version);
  for (const [key, value] of Object.entries(policy)) console.log(`${key}=${value}`);
}
