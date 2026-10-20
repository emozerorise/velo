<template>
  <section v-show="store.isPanelOpen" class="workspace fixed inset-y-0 right-0 z-40 flex pointer-events-auto text-fg" :class="reading ? 'inset-x-0' : 'w-[26rem] max-w-[50vw] border-l border-fg/10 shadow-2xl'" :aria-label="t('workspace.summaryTitle')" @keydown.space.stop>
    <template v-if="reading">
      <button v-if="sidebarOpen" class="absolute inset-0 z-[4] bg-black/50 sidebar-backdrop" :aria-label="t('workspace.closePlaylist')" @click="sidebarOpen = false" />
      <ReadingSidebar :open="sidebarOpen" @close="sidebarOpen = false" />
    </template>
    <div class="workspace-content flex-1 min-w-0 min-h-0 flex flex-col bg-surface">
      <header v-if="reading" data-tauri-drag-region class="workspace-header h-16 shrink-0 flex items-center gap-3 px-6 border-b border-fg/10">
        <button class="sidebar-toggle p-1.5 rounded-lg text-fg/65 hover:text-fg" :aria-label="t('playlist.title')" :aria-expanded="sidebarOpen" @click="sidebarOpen = !sidebarOpen"><PanelLeft class="w-5 h-5" /></button>
        <FileAudio class="w-4 h-4 text-fg/45 shrink-0" aria-hidden="true" />
        <span class="truncate text-sm text-fg/60" :title="currentTitle">{{ currentTitle }}</span>
        <span v-if="player.duration > 0" class="ml-auto font-mono text-xs text-fg/35">{{ formatTime(player.duration) }}</span>
      </header>
      <header v-else class="flex items-center gap-2 px-4 py-3 border-b border-fg/10 shrink-0">
        <FileText class="w-4 h-4 text-accent" aria-hidden="true" />
        <span class="text-sm font-medium text-fg/90">{{ t(workspace.view === 'summary' ? 'workspace.summaryTitle' : 'transcript.title') }}</span>
        <button class="ml-auto p-1.5 rounded-lg text-fg/60 hover:text-fg" :aria-label="t('workspace.fullReading')" @click="store.showReading()"><Maximize2 class="w-4 h-4" /></button>
        <button class="p-1.5 rounded-lg text-fg/60 hover:text-fg" :aria-label="t('summary.closePanel')" @click="store.isPanelOpen = false"><X class="w-4 h-4" /></button>
      </header>
      <div v-if="reading" class="workspace-heading shrink-0 px-6 sm:px-8 pt-6 pb-4">
        <h1 class="text-[30px] font-semibold tracking-tight text-fg/95 pb-4 border-b border-fg/10">{{ t(workspace.view === 'summary' ? 'workspace.summaryTitle' : 'transcript.title') }}</h1>
      </div>
      <!-- Visibility changes must not discard reader search, scroll or drafts. -->
      <div v-show="workspace.view === 'transcript'" id="workspace-transcript" class="workspace-reader flex-1 min-h-0 flex flex-col">
        <TranscriptTab :reading="reading" :visible="workspace.view === 'transcript' && store.isPanelOpen" />
      </div>
      <div v-show="workspace.view === 'summary'" id="workspace-summary" class="workspace-reader flex-1 min-h-0 flex flex-col">
        <WorkspaceWelcome v-if="!player.mediaInfo" />
        <div v-show="!!player.mediaInfo" class="flex-1 min-h-0 flex flex-col">
          <SummaryTab :expanded="reading" :reading="reading" @toggle-reading="reading ? store.showVideo() : store.showReading()" @goto-transcript="workspace.view = 'transcript'" />
        </div>
        <div v-show="workspace.chatVisible" id="workspace-chat" class="border-t border-fg/10 px-6 py-4">
          <slot name="chat">
            <p class="text-sm font-medium text-fg/85">{{ t('workspace.chatTitle') }}</p>
            <p class="mt-1 text-xs text-fg/45">{{ t(player.mediaInfo ? 'workspace.chatUnavailable' : 'summary.openFirst') }}</p>
          </slot>
        </div>
      </div>
      <div v-if="reading && workspace.playerOpen" id="workspace-player" class="shrink-0 border-t border-fg/10">
        <p v-if="workspace.viewportError" role="alert" class="p-3 text-xs text-danger">{{ t('workspace.viewportError') }}</p>
        <ReadingTransport v-if="player.mediaInfo" />
        <p v-else class="p-6 text-sm text-fg/45">{{ t('summary.openFirst') }}</p>
      </div>
    </div>
    <WorkspaceRail />
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { FileAudio, FileText, Maximize2, PanelLeft, X } from '@lucide/vue';
import TranscriptTab from './TranscriptTab.vue';
import SummaryTab from './SummaryTab.vue';
import ReadingSidebar from '@/components/playlist/ReadingSidebar.vue';
import ReadingTransport from '@/components/player/ReadingTransport.vue';
import WorkspaceRail from '@/components/workspace/WorkspaceRail.vue';
import WorkspaceWelcome from '@/components/workspace/WorkspaceWelcome.vue';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { useI18n } from '@/composables/useI18n';
import { formatTime } from '@/utils/formatters';
const store = useTranscriptStore();
const player = usePlayerStore();
const workspace = useWorkspaceStore();
const { t } = useI18n();
const reading = computed(() => workspace.mode === 'reading');
const currentTitle = computed(() => player.mediaInfo?.file_name ?? t('workspace.noFile'));
const sidebarOpen = ref(false);
</script>

<style scoped>
.sidebar-toggle, .sidebar-backdrop { display: none; }
@media (max-width: 899px) {
  .sidebar-toggle, .sidebar-backdrop { display: block; }
  .workspace-header { padding-left: 84px; }
}
@media (max-height: 550px) {
  .workspace-header { height: 48px; }
  .workspace-heading { padding-top: 8px; padding-bottom: 8px; }
  .workspace-heading h1 { font-size: 22px; line-height: 30px; padding-bottom: 8px; }
}
</style>
