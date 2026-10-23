import { beforeEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ingestMedia, playLibraryRecording } from '@/services/mediaIngestion';
import { usePlayerStore } from '@/stores/playerStore';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { emptyLibrary } from '@/types/library';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeEach(() => {
  setActivePinia(createPinia());
  vi.mocked(invoke).mockReset().mockImplementation(async (command, args) => {
    if (command === 'library_load') return emptyLibrary();
    if (command === 'library_media_exists') return true;
    if (command === 'library_mutate') return { ...emptyLibrary(), revision: Number((args as Record<string, unknown>)?.expectedRevision) + 1 };
  });
});

it('imports and selects through the native paused-load boundary without an extra Play command', async () => {
  await ingestMedia(['/a.mp4']);
  await playLibraryRecording('/b.aac');
  expect(vi.mocked(invoke).mock.calls.filter(([c]) => c === 'player_load_file')).toEqual([
    ['player_load_file', { path: '/a.mp4', startTime: undefined }],
    ['player_load_file', { path: '/b.aac', startTime: undefined }],
  ]);
  expect(vi.mocked(invoke).mock.calls.some(([c]) => c === 'player_play' || c === 'player_toggle_pause')).toBe(false);
});

it('seeking and revealing panels do not dispatch Play; explicit Play does', () => {
  const player = usePlayerStore();
  player.state = 'paused';
  const workspace = useWorkspaceStore();
  workspace.togglePlayer(); workspace.toggleChat(); workspace.toggleTranscript();
  player.seekAbsolute(37);
  expect(vi.mocked(invoke).mock.calls).toEqual([['player_seek_absolute', { seconds: 37 }]]);
  player.play();
  expect(vi.mocked(invoke).mock.calls.at(-1)).toEqual(['player_play']);
});
