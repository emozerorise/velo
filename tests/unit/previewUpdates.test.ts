import { afterEach, expect, it, vi } from 'vitest';
const { check, getVersion } = vi.hoisted(() => ({ check: vi.fn(), getVersion: vi.fn() }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion }));
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch: vi.fn() }));
import { updateService } from '@/services/updateService';
afterEach(() => { vi.unstubAllEnvs(); check.mockReset(); });
it('does not invoke the native updater for a stable Windows build', async () => {
  vi.stubEnv('TAURI_ENV_PLATFORM', 'windows');
  getVersion.mockResolvedValue('0.9.0');
  expect(await updateService.check()).toBeNull();
  expect(check).not.toHaveBeenCalled();
});
it('never calls the updater from a prerelease, even for a manual check', async () => {
  getVersion.mockResolvedValue('0.9.0-beta.1');
  expect(await updateService.check()).toBeNull();
  expect(check).not.toHaveBeenCalled();
});
