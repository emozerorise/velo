<template>
  <aside
    class="reading-sidebar bg-chrome border-r border-fg/10 flex flex-col shrink-0"
    :class="{ 'is-open': open }"
    :aria-label="t('playlist.title')"
  >
    <div
      data-tauri-drag-region
      class="h-[84px] shrink-0 flex items-end gap-2 px-5 pb-4"
    >
      <AudioLines class="w-8 h-8 text-accent" aria-hidden="true" />
      <span class="text-[32px] font-semibold tracking-tight text-fg">Velo</span>
      <button
        class="sidebar-close ml-auto p-1 text-fg/60"
        :aria-label="t('workspace.closePlaylist')"
        @click="emit('close')"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
    <div class="flex gap-1 px-4 pb-5">
      <button
        class="flex-1 flex items-center gap-2 rounded-xl border border-fg/15 px-3 py-3 text-base text-fg/90 hover:bg-fg/5"
        @click="openFileDialog"
      >
        <CirclePlus class="w-4 h-4 text-accent" aria-hidden="true" />{{
          t('workspace.openFile')
        }}
      </button>
      <button
        class="px-2 text-fg/55 hover:text-fg rounded-lg"
        :aria-label="t('app.openFolder')"
        :title="t('app.openFolder')"
        @click="openDirectoryDialog"
      >
        <FolderOpen class="w-4 h-4" />
      </button>
    </div>
    <p class="px-5 py-3 border-t border-fg/5 text-sm text-fg/45">
      {{ t('workspace.playlistFiles') }}
    </p>
    <div class="flex-1 min-h-0 overflow-y-auto px-2 space-y-1">
      <div
        v-for="(item, index) in playlist.items"
        :key="item.id"
        class="group flex items-center rounded-xl border-l-2"
        :class="
          index === playlist.currentIndex
            ? 'bg-accent/10 border-accent'
            : 'border-transparent hover:bg-fg/5'
        "
      >
        <button
          class="flex-1 min-w-0 flex items-center gap-3 px-3 py-4 text-left"
          :aria-current="index === playlist.currentIndex ? 'true' : undefined"
          :title="item.fileName"
          @click="
            playlist.playIndex(index);
            emit('close');
          "
        >
          <Play
            v-if="index === playlist.currentIndex"
            class="w-4 h-4 text-accent shrink-0"
            aria-hidden="true"
          />
          <FileAudio
            v-else
            class="w-4 h-4 text-fg/50 shrink-0"
            aria-hidden="true"
          />
          <span class="min-w-0"
            ><span class="block truncate text-[13px] text-fg/85">{{
              item.fileName
            }}</span
            ><span
              v-if="
                index === playlist.currentIndex
                  ? player.duration > 0
                  : item.duration
              "
              class="block text-xs text-fg/40 mt-1 font-mono"
              >{{
                formatTime(
                  index === playlist.currentIndex
                    ? player.duration
                    : (item.duration ?? 0),
                )
              }}</span
            ></span
          >
        </button>
        <button
          class="p-2 mr-1 rounded-lg text-fg/40 opacity-0 group-hover:opacity-100 focus:opacity-100 hover:text-danger"
          :aria-label="`${t('playlist.remove')}: ${item.fileName}`"
          @click="playlist.removeItem(index)"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
      <div
        v-if="!playlist.items.length"
        class="px-4 py-8 text-center space-y-2"
      >
        <FileText class="w-8 h-8 mx-auto mb-3 text-fg/50" aria-hidden="true" />
        <p class="text-base text-fg/85">{{ t('workspace.noFiles') }}</p>
        <p class="text-sm text-fg/45">{{ t('workspace.filesAppearHere') }}</p>
      </div>
    </div>
    <button
      class="flex items-center gap-3 px-5 py-5 text-base text-fg/65 hover:text-fg border-t border-fg/5"
      @click="settings.openSettings()"
    >
      <Settings class="w-4 h-4" aria-hidden="true" />{{ t('topbar.settings') }}
    </button>
  </aside>
</template>

<script setup lang="ts">
import {
  AudioLines,
  CirclePlus,
  FileAudio,
  FileText,
  FolderOpen,
  Play,
  Settings,
  X,
} from '@lucide/vue';
import { usePlaylistStore } from '@/stores/playlistStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useI18n } from '@/composables/useI18n';
import { formatTime } from '@/utils/formatters';

defineProps<{ open: boolean }>();
const emit = defineEmits<{ (event: 'close'): void }>();
const playlist = usePlaylistStore();
const player = usePlayerStore();
const settings = useSettingsStore();
const { openFileDialog, openDirectoryDialog } = useKeyboardShortcuts();
const { t } = useI18n();
</script>

<style scoped>
.reading-sidebar {
  width: 280px;
}
.sidebar-close {
  display: none;
}
@media (max-width: 899px) {
  .reading-sidebar {
    display: none;
  }
  .reading-sidebar.is-open {
    display: flex;
    position: absolute;
    inset: 0 auto 0 0;
    z-index: 5;
    box-shadow: 16px 0 48px rgb(0 0 0 / 0.35);
  }
  .sidebar-close {
    display: block;
  }
}
</style>
