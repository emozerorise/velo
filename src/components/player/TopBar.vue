<template>
  <div
    data-tauri-drag-region
    class="w-full h-14 flex items-center justify-between gap-4 pl-[84px] pr-3 pointer-events-auto"
  >
    <!-- Opaque pills rather than a scrim: readable over any frame without
         veiling the video behind the window. -->
    <div
      class="min-w-0 max-w-[46vw] px-3 py-1.5 rounded-xl bg-[#0e0e12] border border-white/10"
    >
      <span class="block text-[13px] font-medium text-white truncate">
        {{ currentTitle }}
      </span>
    </div>

    <div
      class="flex items-center gap-0.5 p-1 rounded-xl bg-[#0e0e12] border border-white/10"
    >
      <IconButton
        title="Media Information"
        size="md"
        @click="settingsStore.isMediaInfoOpen = true"
      >
        <Info class="w-[18px] h-[18px]" />
      </IconButton>

      <IconButton
        title="Playlist"
        size="md"
        :active="playlistStore.isDrawerOpen"
        @click="playlistStore.toggleDrawer()"
      >
        <ListVideo class="w-[18px] h-[18px]" />
      </IconButton>

      <IconButton
        title="Settings"
        size="md"
        @click="settingsStore.isSettingsOpen = true"
      >
        <Settings class="w-[18px] h-[18px]" />
      </IconButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Info, ListVideo, Settings } from '@lucide/vue';
import IconButton from '@/components/common/IconButton.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useSettingsStore } from '@/stores/settingsStore';

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const settingsStore = useSettingsStore();

const currentTitle = computed(() => {
  if (playerStore.mediaInfo?.file_name) {
    return playerStore.mediaInfo.file_name;
  }
  if (playlistStore.currentItem?.fileName) {
    return playlistStore.currentItem.fileName;
  }
  return 'Velo';
});
</script>
