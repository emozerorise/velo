<template>
  <div
    class="relative w-full h-full overflow-hidden select-none bg-transparent"
    :class="{ 'cursor-none': !areControlsVisible && playerStore.state === 'playing' }"
  >
    <!-- Toast HUD -->
    <ToastStack />

    <!-- TopBar (auto-hiding) -->
    <Transition name="fade-bar">
      <TopBar
        v-if="areControlsVisible || playerStore.state !== 'playing'"
        class="fixed top-0 inset-x-0 z-30"
      />
    </Transition>

    <!-- Center Video Interaction / Gesture Layer (Clicks on empty area) -->
    <div
      class="absolute inset-0 z-10"
      @click="onVideoClick"
      @dblclick="toggleFullscreen"
    />

    <!-- Empty state -->
    <div
      v-if="!hasMedia && playlistStore.items.length === 0"
      class="absolute inset-0 z-20 flex flex-col items-center justify-center pointer-events-none p-6"
    >
      <div class="pointer-events-auto w-full max-w-[320px] flex flex-col items-center text-center">
        <div
          class="w-14 h-14 mb-6 rounded-2xl bg-blue-500 flex items-center justify-center shadow-[0_10px_36px_-10px_rgba(59,130,246,0.9)]"
        >
          <Play class="w-6 h-6 fill-current text-white translate-x-[1px]" />
        </div>

        <h1 class="text-[22px] font-semibold tracking-tight text-fg">Velo</h1>
        <p class="mt-1.5 text-[13px] leading-relaxed text-fg/45">
          {{ t('app.tagline') }}
        </p>

        <div class="mt-7 w-full flex flex-col gap-2">
          <button
            class="w-full h-10 px-4 rounded-xl bg-blue-500 hover:bg-blue-400 active:scale-[0.98] text-white text-[13px] font-semibold transition-all flex items-center justify-center gap-2 shadow-[0_8px_28px_-10px_rgba(59,130,246,0.9)]"
            @click="openFileDialog"
          >
            <FolderOpen class="w-4 h-4" />
            <span>{{ t('app.openVideo') }}</span>
          </button>

          <button
            class="w-full h-10 px-4 rounded-xl bg-fg/[0.07] hover:bg-fg/[0.12] active:scale-[0.98] text-fg/85 text-[13px] font-medium transition-all flex items-center justify-center gap-2"
            @click="openDirectoryDialog"
          >
            <FolderArchive class="w-4 h-4" />
            <span>{{ t('app.openFolder') }}</span>
          </button>
        </div>

        <div class="mt-7 flex items-center gap-3 text-[11px] text-fg/30">
          <span class="flex items-center gap-1.5">
            <kbd class="px-1.5 py-0.5 rounded border border-fg/15 font-mono">Space</kbd>
            {{ t('app.hint.play') }}
          </span>
          <span class="flex items-center gap-1.5">
            <kbd class="px-1.5 py-0.5 rounded border border-fg/15 font-mono">⌘O</kbd>
            {{ t('app.hint.open') }}
          </span>
          <span class="flex items-center gap-1.5">
            <kbd class="px-1.5 py-0.5 rounded border border-fg/15 font-mono">F</kbd>
            {{ t('app.hint.full') }}
          </span>
        </div>
      </div>
    </div>

    <!-- Bottom Controls Overlay (auto-hiding) -->
    <Transition name="fade-bar">
      <div
        v-if="areControlsVisible || playerStore.state !== 'playing'"
        class="fixed bottom-0 inset-x-0 z-30"
      >
        <ControlOverlay />
      </div>
    </Transition>

    <!-- Side Drawer: Playlist -->
    <PlaylistDrawer />

    <!-- Side Drawer: Transcript -->
    <TranscriptPanel />

    <!-- Modals -->
    <SettingsModal />
    <MediaInfoModal />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue';
import { Play, FolderOpen, FolderArchive } from '@lucide/vue';
import TopBar from '@/components/player/TopBar.vue';
import ControlOverlay from '@/components/player/ControlOverlay.vue';
import PlaylistDrawer from '@/components/playlist/PlaylistDrawer.vue';
import TranscriptPanel from '@/components/transcript/TranscriptPanel.vue';
import SettingsModal from '@/components/settings/SettingsModal.vue';
import MediaInfoModal from '@/components/player/MediaInfoModal.vue';
import ToastStack from '@/components/common/ToastStack.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useAutoHideControls } from '@/composables/useAutoHideControls';
import { usePlaybackHistory } from '@/composables/usePlaybackHistory';
import { useI18n } from '@/composables/useI18n';

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const settingsStore = useSettingsStore();
const transcriptStore = useTranscriptStore();

const { openFileDialog, openDirectoryDialog, toggleFullscreen } = useKeyboardShortcuts();
const { areControlsVisible } = useAutoHideControls();
const { t } = useI18n();
usePlaybackHistory();

let clickTimeout: number | null = null;

function onVideoClick() {
  if (clickTimeout) {
    clearTimeout(clickTimeout);
    clickTimeout = null;
    return;
  }

  clickTimeout = window.setTimeout(() => {
    playerStore.togglePlay();
    clickTimeout = null;
  }, 200);
}

// The video is a native surface *behind* the webview, so the page has to go
// transparent for it to be visible. Keyed off media presence as well as
// state, so a dropped state event cannot leave the video hidden for good.
const hasMedia = computed(
  () => playerStore.state !== 'idle' || playerStore.mediaInfo !== null || playerStore.duration > 0
);

watch(
  hasMedia,
  (active) => {
    document.documentElement.classList.toggle('video-surface-active', active);
  },
  { immediate: true }
);

// A transcript belongs to one file, so swapping files swaps the panel's
// contents -- cached ones load instantly, the rest show the generate prompt.
watch(
  () => playerStore.mediaInfo?.path,
  () => {
    void transcriptStore.loadForCurrentMedia();
  }
);

onMounted(async () => {
  await playerStore.initListeners();
  await transcriptStore.initListeners();
  await settingsStore.loadSettings();
});

onUnmounted(() => {
  document.documentElement.classList.remove('video-surface-active');
  playerStore.cleanupListeners();
  transcriptStore.cleanupListeners();
});
</script>

<style scoped>
.fade-bar-enter-active,
.fade-bar-leave-active {
  transition: opacity 0.25s cubic-bezier(0.16, 1, 0.3, 1), transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.fade-bar-enter-from,
.fade-bar-leave-to {
  opacity: 0;
}
</style>
