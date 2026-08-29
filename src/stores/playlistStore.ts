import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { PlaylistItem, LoopMode } from '@/types/playlist';
import { usePlayerStore } from './playerStore';

export const usePlaylistStore = defineStore('playlist', () => {
  const items = ref<PlaylistItem[]>([]);
  const currentIndex = ref<number>(-1);
  const loopMode = ref<LoopMode>('off');
  const shuffle = ref<boolean>(false);
  const isDrawerOpen = ref<boolean>(false);

  const currentItem = computed(() => {
    if (currentIndex.value >= 0 && currentIndex.value < items.value.length) {
      return items.value[currentIndex.value];
    }
    return null;
  });

  const hasNext = computed(() => {
    if (items.value.length === 0) return false;
    if (loopMode.value === 'all' || loopMode.value === 'single') return true;
    return currentIndex.value < items.value.length - 1;
  });

  const hasPrevious = computed(() => {
    if (items.value.length === 0) return false;
    if (loopMode.value === 'all') return true;
    return currentIndex.value > 0;
  });

  function addFiles(paths: string[]) {
    for (const path of paths) {
      const fileName = path.split('/').pop()?.split('\\').pop() || path;
      // Avoid duplicate addition of exact same path
      if (!items.value.some((item) => item.path === path)) {
        items.value.push({
          id: crypto.randomUUID(),
          path,
          fileName,
        });
      }
    }

    if (currentIndex.value === -1 && items.value.length > 0) {
      playIndex(0);
    }
  }

  async function addDirectory(dirPath: string) {
    try {
      const res = await invoke<{ files: string[] }>('playlist_scan_directory', {
        dirPath,
      });
      if (res && res.files && res.files.length > 0) {
        addFiles(res.files);
      }
    } catch (e) {
      console.error('Failed to scan directory:', e);
    }
  }

  function removeItem(index: number) {
    if (index < 0 || index >= items.value.length) return;
    items.value.splice(index, 1);
    if (currentIndex.value >= items.value.length) {
      currentIndex.value = items.value.length - 1;
    }
  }

  function clear() {
    items.value = [];
    currentIndex.value = -1;
  }

  function playIndex(index: number) {
    if (index < 0 || index >= items.value.length) return;
    currentIndex.value = index;
    const item = items.value[index];
    const player = usePlayerStore();
    player.loadFile(item.path, item.lastPosition);
  }

  function next() {
    if (items.value.length === 0) return;

    if (loopMode.value === 'single') {
      playIndex(currentIndex.value);
      return;
    }

    if (shuffle.value && items.value.length > 1) {
      let rand = currentIndex.value;
      while (rand === currentIndex.value) {
        rand = Math.floor(Math.random() * items.value.length);
      }
      playIndex(rand);
      return;
    }

    if (currentIndex.value < items.value.length - 1) {
      playIndex(currentIndex.value + 1);
    } else if (loopMode.value === 'all') {
      playIndex(0);
    }
  }

  function previous() {
    if (items.value.length === 0) return;

    const player = usePlayerStore();
    // If playing for more than 3 seconds, restart current track
    if (player.currentTime > 3) {
      player.seekAbsolute(0);
      return;
    }

    if (currentIndex.value > 0) {
      playIndex(currentIndex.value - 1);
    } else if (loopMode.value === 'all') {
      playIndex(items.value.length - 1);
    }
  }

  function toggleDrawer() {
    isDrawerOpen.value = !isDrawerOpen.value;
  }

  return {
    items,
    currentIndex,
    loopMode,
    shuffle,
    isDrawerOpen,
    currentItem,
    hasNext,
    hasPrevious,
    addFiles,
    addDirectory,
    removeItem,
    clear,
    playIndex,
    next,
    previous,
    toggleDrawer,
  };
});
