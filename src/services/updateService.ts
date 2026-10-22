import { getVersion } from '@tauri-apps/api/app';
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateOffer = Pick<Update, 'version' | 'body' | 'download' | 'install' | 'close'>;
export type UpdateDownloadEvent = DownloadEvent;

/** The native plugins own HTTP, signature verification and app replacement. */
export const updateService = {
  get manualOnly(): boolean {
    return import.meta.env.TAURI_ENV_PLATFORM === 'windows';
  },
  currentVersion: getVersion,
  check: async (): Promise<UpdateOffer | null> => {
    if (updateService.manualOnly) return null;
    // Test builds never participate in the production update channel.
    if ((await getVersion()).includes('-')) return null;
    return check({ timeout: 10_000 });
  },
  relaunch,
};
