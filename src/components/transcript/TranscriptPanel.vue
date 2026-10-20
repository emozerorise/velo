<template>
  <section
    v-show="store.isPanelOpen"
    class="fixed inset-y-0 right-0 z-40 flex pointer-events-auto bg-surface text-fg"
    :class="
      store.isReadingMode ? 'inset-x-0' : 'w-[26rem] max-w-[50vw] border-l border-fg/10 shadow-2xl'
    "
    :aria-label="t('transcript.title')"
    @keydown.space.stop
  >
    <template v-if="store.isReadingMode">
      <button
        v-if="sidebarOpen"
        class="absolute inset-0 z-[4] bg-black/50 sidebar-backdrop"
        :aria-label="t('workspace.closePlaylist')"
        @click="sidebarOpen = false"
      />
      <ReadingSidebar :open="sidebarOpen" @close="sidebarOpen = false" />
    </template>
    <div class="flex-1 min-w-0 min-h-0 flex flex-col">
      <header
        v-if="store.isReadingMode"
        data-tauri-drag-region
        class="workspace-header h-[64px] shrink-0 flex items-center gap-3 px-5 border-b border-fg/10"
      >
        <button
          class="sidebar-toggle p-1.5 rounded-lg text-fg/65 hover:text-fg"
          :aria-label="t('playlist.title')"
          :aria-expanded="sidebarOpen"
          @click="sidebarOpen = !sidebarOpen"
        >
          <PanelLeft class="w-5 h-5" />
        </button>
        <FileAudio class="w-4 h-4 text-fg/55 shrink-0 hidden sm:block" aria-hidden="true" />
        <span class="truncate text-sm text-fg/80" :title="currentTitle">{{ currentTitle }}</span>
        <button
          class="ml-auto shrink-0 flex items-center gap-2 px-4 py-2 rounded-xl bg-blue-500 hover:bg-blue-400 text-slate-950 text-sm font-semibold"
          :title="t('workspace.videoHint')"
          @click="store.showVideo()"
          @keydown.space.stop.prevent="store.showVideo()"
        >
          <MonitorPlay class="w-4 h-4" aria-hidden="true" />{{ t('workspace.watchVideo') }}
        </button>
      </header>
      <header v-else class="flex items-center gap-2 px-4 py-3 border-b border-fg/10 shrink-0">
        <Captions class="w-4 h-4 text-accent" aria-hidden="true" />
        <span class="text-sm font-medium text-fg/90">{{ t('transcript.title') }}</span>
        <button
          class="ml-auto p-1.5 rounded-lg text-fg/60 hover:text-fg hover:bg-fg/5"
          :aria-label="t('workspace.fullReading')"
          :title="t('workspace.fullReading')"
          @click="store.showReading()"
          @keydown.space.stop.prevent="store.showReading()"
        >
          <Maximize2 class="w-4 h-4" />
        </button>
        <button
          class="p-1.5 rounded-lg text-fg/60 hover:text-fg hover:bg-fg/5"
          :aria-label="t('summary.closePanel')"
          @click="store.isPanelOpen = false"
        >
          <X class="w-4 h-4" />
        </button>
      </header>

      <div
        class="shrink-0 workspace-heading"
        :class="store.isReadingMode ? 'px-6 pt-7 sm:px-8' : 'px-3 pt-2'"
      >
        <div :class="store.isReadingMode ? 'max-w-[58rem] mx-auto' : ''">
          <template v-if="store.isReadingMode">
            <h1 class="text-[32px] font-semibold tracking-tight text-fg/95">
              {{ t('workspace.readTranscript') }}
            </h1>
            <p class="workspace-meta mt-2 mb-5 text-sm text-fg/40">
              {{ store.transcript?.language ?? t('workspace.ready')
              }}<template v-if="player.duration > 0"> · {{ formatTime(player.duration) }}</template>
            </p>
          </template>
          <div
            class="workspace-tabs flex items-center gap-6 border-b border-fg/10"
            role="tablist"
            :aria-label="t('transcript.title')"
          >
            <button
              v-for="option in TABS"
              :id="`reading-tab-${option.id}`"
              :key="option.id"
              role="tab"
              :aria-selected="tab === option.id"
              :aria-controls="`reading-panel-${option.id}`"
              class="py-3 text-sm font-medium border-b-2 transition-colors"
              :class="
                tab === option.id
                  ? 'text-accent border-accent'
                  : 'text-fg/45 hover:text-fg border-transparent'
              "
              @click="tab = option.id"
              @keydown.space.stop.prevent="tab = option.id"
            >
              {{ t(option.key) }}
            </button>
          </div>
        </div>
      </div>

      <!-- Keep both readers mounted: switching view must not throw away the
           search, scroll position, expanded sections or an in-progress draft. -->
      <div
        v-show="tab === 'transcript'"
        id="reading-panel-transcript"
        role="tabpanel"
        aria-labelledby="reading-tab-transcript"
        class="flex-1 min-h-0 flex flex-col"
      >
        <TranscriptTab
          :reading="store.isReadingMode"
          :visible="tab === 'transcript' && store.isPanelOpen"
        />
      </div>
      <div
        v-show="tab === 'summary'"
        id="reading-panel-summary"
        role="tabpanel"
        aria-labelledby="reading-tab-summary"
        class="flex-1 min-h-0 flex flex-col"
      >
        <SummaryTab
          :expanded="store.isReadingMode"
          :reading="store.isReadingMode"
          @toggle-reading="store.isReadingMode ? store.showVideo() : store.showReading()"
          @goto-transcript="tab = 'transcript'"
        />
      </div>
      <ReadingTransport v-if="store.isReadingMode && player.mediaInfo" />
      <div v-else-if="store.isReadingMode" class="shrink-0 px-6 py-6 flex justify-center">
        <button
          class="flex items-center gap-2 px-5 py-3 rounded-xl bg-blue-500 hover:bg-blue-400 text-white text-sm font-medium"
          @click="openFileDialog"
        >
          <FolderOpen class="w-4 h-4" aria-hidden="true" />{{ t('app.openMedia') }}
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Captions, FileAudio, FolderOpen, Maximize2, MonitorPlay, PanelLeft, X } from '@lucide/vue';
import TranscriptTab from './TranscriptTab.vue';
import SummaryTab from './SummaryTab.vue';
import ReadingSidebar from '@/components/playlist/ReadingSidebar.vue';
import ReadingTransport from '@/components/player/ReadingTransport.vue';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useI18n } from '@/composables/useI18n';
import { formatTime } from '@/utils/formatters';

const store = useTranscriptStore();
const player = usePlayerStore();
const playlist = usePlaylistStore();
const { t } = useI18n();
const { openFileDialog } = useKeyboardShortcuts();
const currentTitle = computed(
  () => player.mediaInfo?.file_name ?? playlist.currentItem?.fileName ?? 'Velo',
);
const TABS = [
  { id: 'transcript', key: 'summary.tab.transcript' },
  { id: 'summary', key: 'summary.tab.summary' },
] as const;
const tab = ref<(typeof TABS)[number]['id']>('transcript');
const sidebarOpen = ref(false);
</script>

<style scoped>
.sidebar-toggle,
.sidebar-backdrop {
  display: none;
}
@media (max-width: 899px) {
  .sidebar-toggle,
  .sidebar-backdrop {
    display: block;
  }
  .workspace-header {
    padding-left: 84px;
  }
}
@media (max-height: 550px) {
  .workspace-header {
    height: 48px;
  }
  .workspace-heading {
    padding-top: 8px;
  }
  .workspace-heading h1 {
    font-size: 20px;
    line-height: 28px;
  }
  .workspace-meta {
    display: none;
  }
  .workspace-tabs button {
    padding-top: 6px;
    padding-bottom: 6px;
  }
}
</style>
