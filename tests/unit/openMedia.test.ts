import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { createPinia, setActivePinia } from 'pinia';

const { open, invoke } = vi.hoisted(() => ({ open: vi.fn(), invoke: vi.fn() }));

vi.mock('@tauri-apps/plugin-dialog', () => ({ open }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { usePlaylistStore } from '@/stores/playlistStore';

async function fileDialog() {
  let shortcuts!: ReturnType<typeof useKeyboardShortcuts>;
  await renderToString(
    createSSRApp({
      setup() {
        shortcuts = useKeyboardShortcuts();
        return () => null;
      },
    }),
  );
  return shortcuts;
}

describe('opening audio and video files', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    open.mockReset().mockResolvedValue(null);
    invoke.mockReset().mockResolvedValue(null);
  });

  it('allows common audio formats alongside video in the native picker', async () => {
    await (await fileDialog()).openFileDialog();

    expect(open).toHaveBeenCalledWith(
      expect.objectContaining({
        multiple: true,
        filters: [
          expect.objectContaining({
            extensions: expect.arrayContaining([
              'mp3',
              'wav',
              'm4a',
              'aac',
              'flac',
              'ogg',
              'opus',
              'mp4',
              'mkv',
              'mov',
            ]),
          }),
        ],
      }),
    );
  });

  it('queues a mixed selection and starts the selected audio file', async () => {
    open.mockResolvedValue(['/meetings/recording.mp3', '/meetings/video.mp4']);

    await (await fileDialog()).openFileDialog();

    expect(usePlaylistStore().items.map((item) => item.path)).toEqual([
      '/meetings/recording.mp3',
      '/meetings/video.mp4',
    ]);
    expect(invoke).toHaveBeenCalledWith('player_load_file', {
      path: '/meetings/recording.mp3',
      startTime: undefined,
    });
  });

  it('leaves the playlist unchanged when the picker is cancelled', async () => {
    await (await fileDialog()).openFileDialog();

    expect(usePlaylistStore().items).toEqual([]);
    expect(invoke).not.toHaveBeenCalled();
  });
});
