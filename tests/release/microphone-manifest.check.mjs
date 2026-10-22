import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const descriptions = {
  en: 'Velo uses the microphone to turn your spoken question into editable text on this device. Audio is not uploaded.',
  th: 'Velo ใช้ไมโครโฟนเพื่อแปลงคำถามที่พูดเป็นข้อความให้ตรวจแก้ในเครื่อง โดยไม่อัปโหลดเสียง',
};
const read = (path) => readFileSync(`src-tauri/${path}`, 'utf8');

test('saved microphone declaration contains the approved local-only explanation', () => {
  assert.equal(config.bundle.macOS.infoPlist, 'Info.plist');
  const plist = read(config.bundle.macOS.infoPlist);
  const strings = Object.fromEntries([...plist.matchAll(/<key>([^<]+)<\/key>\s*<string>([^<]*)<\/string>/g)].map((m) => [m[1], m[2]]));
  assert.equal(strings.NSMicrophoneUsageDescription, descriptions.en);
  assert.doesNotMatch(plist, /NSCamera|ScreenCapture|SystemAudio|AudioCapture/i);
});
test('both localized declarations are mapped into the app Contents/Resources', () => {
  for (const locale of ['en', 'th']) {
    const source = `resources/${locale}.lproj/InfoPlist.strings`;
    assert.equal(config.bundle.macOS.files?.[`Resources/${locale}.lproj/InfoPlist.strings`], source);
    const contents = read(source);
    const entry = contents.match(/^\s*"NSMicrophoneUsageDescription"\s*=\s*("(?:[^"\\]|\\.)*");\s*$/u);
    assert.ok(entry, `valid ${locale} InfoPlist.strings`);
    assert.equal(JSON.parse(entry[1]), descriptions[locale]);
  }
});
test('permission packaging preserves the OS floor, version and security policy', () => {
  assert.equal(config.bundle.macOS.minimumSystemVersion, '11.0');
  assert.equal(config.version, JSON.parse(readFileSync('package.json', 'utf8')).version);
  assert.equal(config.identifier, 'com.velo.player');
  assert.equal(config.app.security.csp, "default-src 'self'; img-src 'self' data: asset: blob:; style-src 'self' 'unsafe-inline'; font-src 'self' data:;");
  assert.deepEqual(Object.keys(config.bundle.macOS.files ?? {}).sort(), ['Resources/en.lproj/InfoPlist.strings', 'Resources/th.lproj/InfoPlist.strings']);
  assert.equal(config.bundle.resources, undefined);
});
