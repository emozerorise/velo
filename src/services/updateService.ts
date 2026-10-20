import { getVersion } from '@tauri-apps/api/app';
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateOffer = Pick<Update, 'version' | 'body' | 'download' | 'install' | 'close'>;
export type UpdateDownloadEvent = DownloadEvent;

/** The native plugins own HTTP, signature verification and app replacement. */
export const updateService = {
  currentVersion: getVersion,
  check: (): Promise<UpdateOffer | null> => check({ timeout: 10_000 }),
  relaunch,
};
