<template>
  <Transition name="slide">
    <div
      v-if="playlistStore.isDrawerOpen"
      class="fixed inset-y-0 right-0 z-40 w-80 bg-surface border-l border-fg/10 flex flex-col pointer-events-auto"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3.5 border-b border-fg/10">
        <div class="flex items-center gap-2">
          <ListVideo class="w-4 h-4 text-accent" />
          <span class="text-sm font-medium text-fg/90">{{ t('playlist.title') }}</span>
          <span class="text-xs text-fg/40 font-mono">({{ playlistStore.items.length }})</span>
        </div>

        <button
          class="p-1 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
          @click="playlistStore.isDrawerOpen = false"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Controls & Action Toolbar -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-fg/5 bg-inset/20 text-xs">
        <div class="flex items-center gap-1">
          <button
            class="px-2 py-1 rounded bg-blue-600/30 hover:bg-blue-600/40 text-accent font-medium transition-colors"
            @click="openFileDialog"
          >
            {{ t('playlist.addFile') }}
          </button>
          <button
            class="px-2 py-1 rounded bg-fg/10 hover:bg-fg/15 text-fg/80 transition-colors"
            @click="openDirectoryDialog"
          >
            {{ t('playlist.addFolder') }}
          </button>
        </div>

        <div class="flex items-center gap-1">
          <!-- Shuffle -->
          <button
            :class="[
              'p-1.5 rounded transition-colors',
              playlistStore.shuffle ? 'text-accent bg-blue-600/20' : 'text-fg/40 hover:text-fg',
            ]"
            :title="t('playlist.shuffle')"
            @click="playlistStore.shuffle = !playlistStore.shuffle"
          >
            <Shuffle class="w-3.5 h-3.5" />
          </button>

          <!-- Loop Mode -->
          <button
            :class="[
              'p-1.5 rounded transition-colors',
              playlistStore.loopMode !== 'off' ? 'text-accent bg-blue-600/20' : 'text-fg/40 hover:text-fg',
            ]"
            :title="t('playlist.loop', { mode: playlistStore.loopMode })"
            @click="toggleLoopMode"
          >
            <Repeat1 v-if="playlistStore.loopMode === 'single'" class="w-3.5 h-3.5" />
            <Repeat v-else class="w-3.5 h-3.5" />
          </button>

          <!-- Clear -->
          <button
            v-if="playlistStore.items.length > 0"
            class="p-1.5 rounded text-fg/40 hover:text-danger transition-colors"
            :title="t('playlist.clear')"
            @click="playlistStore.clear()"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- Item List -->
      <div class="flex-1 p-3 overflow-y-auto flex flex-col gap-1">
        <PlaylistItem
          v-for="(item, index) in playlistStore.items"
          :key="item.id"
          :item="item"
          :index="index"
          :is-active="index === playlistStore.currentIndex"
          @play="playlistStore.playIndex(index)"
          @remove="playlistStore.removeItem(index)"
        />

        <div
          v-if="playlistStore.items.length === 0"
          class="h-full flex flex-col items-center justify-center text-center p-6 text-fg/40 gap-2"
        >
          <FolderOpen class="w-8 h-8 opacity-40" />
          <p class="text-xs">{{ t('playlist.empty') }}</p>
          <p class="text-[10px] text-fg/30">{{ t('playlist.emptyHint') }}</p>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import {
  ListVideo,
  X,
  Shuffle,
  Repeat,
  Repeat1,
  Trash2,
  FolderOpen,
} from '@lucide/vue';
import PlaylistItem from '@/components/playlist/PlaylistItem.vue';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useI18n } from '@/composables/useI18n';

const playlistStore = usePlaylistStore();
const { t } = useI18n();
const { openFileDialog, openDirectoryDialog } = useKeyboardShortcuts();

function toggleLoopMode() {
  if (playlistStore.loopMode === 'off') {
    playlistStore.loopMode = 'all';
  } else if (playlistStore.loopMode === 'all') {
    playlistStore.loopMode = 'single';
  } else {
    playlistStore.loopMode = 'off';
  }
}
</script>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
