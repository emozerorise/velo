import { invoke } from '@tauri-apps/api/core';
import { useLibraryStore } from '@/stores/libraryStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useToast } from '@/composables/useToast';
import { useI18n } from '@/composables/useI18n';

export async function ingestMedia(paths: string[], projectId: string | null = null): Promise<void> {
  if (!paths.length) return;
  const saved = await useLibraryStore().apply({type: 'add_recordings', entries: paths.map(path => ({id: crypto.randomUUID(), path})), project_id: projectId});
  if (!saved) useToast().showToast(useI18n().t('library.unsaved'));
  usePlaylistStore().addFiles(paths);
}
export async function ingestDirectory(path: string, projectId: string | null = null): Promise<void> {
  const result = await invoke<{files: string[]}>('playlist_scan_directory', {dirPath: path});
  await ingestMedia(result.files, projectId);
}
export async function ingestDroppedPaths(paths: string[]): Promise<void> {
  const result = await invoke<{files: string[]}>('playlist_resolve_paths', {paths});
  await ingestMedia(result.files);
}
export async function playLibraryRecording(path: string): Promise<boolean> {
  if (!await useLibraryStore().checkAvailable(path)) return false;
  const queue = usePlaylistStore();
  queue.ensureFiles([path]);
  queue.playIndex(queue.items.findIndex(r => r.path === path));
  return true;
}
