import { expect, it, vi } from 'vitest';
const { check, getVersion } = vi.hoisted(() => ({ check: vi.fn(), getVersion: vi.fn() }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion }));
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch: vi.fn() }));
import { updateService } from '@/services/updateService';
it('never calls the updater from a prerelease, even for a manual check', async () => {
  getVersion.mockResolvedValue('0.9.0-beta.1');
  expect(await updateService.check()).toBeNull();
  expect(check).not.toHaveBeenCalled();
});
