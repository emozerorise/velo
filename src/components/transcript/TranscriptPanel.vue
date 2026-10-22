<template>
  <section
    v-show="store.isPanelOpen"
    class="workspace fixed inset-y-0 right-0 z-40 flex pointer-events-auto text-fg"
    :class="
      reading
        ? 'inset-x-0'
        : 'w-[26rem] max-w-[50vw] border-l border-fg/10 shadow-2xl'
    "
    :aria-label="t('workspace.summaryTitle')"
    @keydown.space.stop
  >
    <template v-if="reading">
      <button
        v-if="sidebarOpen"
        class="absolute inset-0 z-[4] bg-black/50 sidebar-backdrop"
        :aria-label="t('workspace.closePlaylist')"
        @click="sidebarOpen = false"
      />
      <ReadingSidebar :open="sidebarOpen" @close="sidebarOpen = false" />
    </template>
    <div class="workspace-content flex-1 min-w-0 min-h-0 flex flex-col">
      <header
        v-if="reading"
        data-tauri-drag-region
        class="workspace-header h-[70px] shrink-0 flex items-center gap-3 px-6 border-b border-fg/10 bg-surface"
      >
        <button
          ref="sidebarToggle"
          class="sidebar-toggle p-1.5 rounded-lg text-fg/65 hover:text-fg"
          :aria-label="t('playlist.title')"
          :aria-expanded="sidebarOpen"
          @click="toggleSidebar"
        >
          <PanelLeft class="w-5 h-5" />
        </button>
        <FileAudio class="w-4 h-4 text-fg/45 shrink-0" aria-hidden="true" />
        <span
          class="workspace-file-title truncate text-base text-fg/60"
          :title="currentTitle"
          >{{ currentTitle }}</span
        >
        <h1 class="compact-view-title truncate text-base font-semibold">
          {{
            t(
              workspace.view === 'summary'
                ? 'workspace.summaryTitle'
                : 'transcript.title',
            )
          }}
        </h1>
        <span
          v-if="player.duration > 0"
          class="ml-auto font-mono text-xs text-fg/35"
          >{{ formatTime(player.duration) }}</span
        >
      </header>
      <header
        v-else
        class="flex items-center gap-2 px-4 py-3 border-b border-fg/10 shrink-0 bg-surface"
      >
        <FileText class="w-4 h-4 text-accent" aria-hidden="true" />
        <span class="text-sm font-medium text-fg/90">{{
          t(
            workspace.view === 'summary'
              ? 'workspace.summaryTitle'
              : 'transcript.title',
          )
        }}</span>
        <button
          class="ml-auto p-1.5 rounded-lg text-fg/60 hover:text-fg"
          :aria-label="t('workspace.fullReading')"
          @click="store.showReading()"
        >
          <Maximize2 class="w-4 h-4" />
        </button>
        <button
          class="p-1.5 rounded-lg text-fg/60 hover:text-fg"
          :aria-label="t('summary.closePanel')"
          @click="store.isPanelOpen = false"
        >
          <X class="w-4 h-4" />
        </button>
      </header>
      <div
        v-if="
          reading &&
          (workspace.view !== 'summary' ||
            !summaries.hasSummary ||
            !player.mediaInfo)
        "
        class="workspace-heading shrink-0 px-6 sm:px-8 pt-6 pb-4 bg-surface"
      >
        <h1
          class="text-[40px] leading-[48px] font-semibold tracking-tight text-fg/95 pb-4 border-b border-fg/10"
        >
          {{
            t(
              workspace.view === 'summary'
                ? 'workspace.summaryTitle'
                : 'transcript.title',
            )
          }}
        </h1>
      </div>
      <!-- Visibility changes must not discard reader search, scroll or drafts. -->
      <div
        v-show="workspace.view === 'transcript'"
        id="workspace-transcript"
        class="workspace-reader flex-1 min-h-0 flex flex-col bg-surface"
        @keydown.esc.stop="closeTool('transcript', $event)"
      >
        <TranscriptTab
          :reading="reading"
          :visible="workspace.view === 'transcript' && store.isPanelOpen"
        />
      </div>
      <div
        v-show="workspace.view === 'summary'"
        id="workspace-summary"
        class="workspace-reader flex-1 min-h-0 flex flex-col bg-surface"
      >
        <WorkspaceWelcome v-if="!player.mediaInfo" ref="welcome" />
        <p v-if="!player.mediaInfo" class="sr-only" aria-live="polite">
          {{ welcomeAnnouncement }}
        </p>
        <div v-show="!!player.mediaInfo" class="flex-1 min-h-0 flex flex-col">
          <SummaryTab
            :expanded="reading"
            :reading="reading"
            @toggle-reading="reading ? store.showVideo() : store.showReading()"
            @goto-transcript="workspace.view = 'transcript'"
          >
            <template #default="{ scrollElement }">
              <div v-if="workspace.chatVisible && historyContext" class="summary-history">
                <button type="button" data-testid="summary-chat-toggle" :aria-expanded="!chatFolded" aria-controls="workspace-chat" @click="chatFolded = !chatFolded">
                  <MessageCircle class="w-5 h-5 shrink-0 text-accent" aria-hidden="true" />
                  <span class="history-copy"><strong>{{ t('summary.previousChat') }}</strong><span>{{ t('summary.previousChatNote') }}</span></span>
                  <span class="history-action">{{ t(chatFolded ? 'summary.openChat' : 'summary.foldChat') }}</span>
                  <ChevronDown class="w-4 h-4 shrink-0" :class="{ 'rotate-180': !chatFolded }" aria-hidden="true" />
                </button>
              </div>
              <InlineChat
                v-show="workspace.chatVisible && !chatFolded && !!player.mediaInfo"
                :visible="workspace.chatVisible && !chatFolded && !!player.mediaInfo"
                :scroll-element="scrollElement"
                @close="closeChat"
                @transcript="workspace.view = 'transcript'"
                @keydown.esc.stop="closeTool('chat', $event)"
              />
            </template>
          </SummaryTab>
        </div>
      </div>
      <PlayerShelf :video-element="videoElement" @close="focusTool('player')" />
    </div>
    <WorkspaceRail ref="rail" @welcome-chat="focusWelcome" />
  </section>
</template>

<script setup lang="ts">
import { useLibraryStore } from '@/stores/libraryStore';
const library = useLibraryStore();
import { computed, nextTick, ref, watch } from 'vue';
import { ChevronDown, MessageCircle, FileAudio, FileText, Maximize2, PanelLeft, X } from '@lucide/vue';
import TranscriptTab from './TranscriptTab.vue';
import SummaryTab from './SummaryTab.vue';
import ReadingSidebar from '@/components/playlist/ReadingSidebar.vue';
import PlayerShelf from '@/components/player/PlayerShelf.vue';
import WorkspaceRail from '@/components/workspace/WorkspaceRail.vue';
import WorkspaceWelcome from '@/components/workspace/WorkspaceWelcome.vue';
import InlineChat from '@/components/chat/InlineChat.vue';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { useSummaryStore } from '@/stores/summaryStore';
import { useI18n } from '@/composables/useI18n';
import { formatTime } from '@/utils/formatters';
import { toolMayClose } from '@/utils/toolKeyboard';
import { useWorkspaceLayout } from '@/composables/useWorkspaceLayout';
import { useVideoMask } from '@/composables/useVideoMask';
import type { WorkspaceTool } from '@/types/workspace';
const store = useTranscriptStore();
const player = usePlayerStore();
const workspace = useWorkspaceStore();
const summaries = useSummaryStore();
const chatFolded = ref(summaries.isRunning);
const historyContext = ref(summaries.isRunning);
watch(() => player.mediaInfo?.path, () => {
  chatFolded.value = false;
  historyContext.value = false;
});
watch(() => workspace.chatVisible, (visible) => {
  if (visible) chatFolded.value = false;
});
// Keep the mounted chat and draft intact while a new summary takes focus.
watch(() => summaries.isRunning, (running) => {
  if (running) {
    chatFolded.value = true;
    historyContext.value = true;
  }
});
const { t } = useI18n();
const reading = computed(() => workspace.mode === 'reading');
const currentTitle = computed(
  () => player.mediaInfo?.path ? library.displayName(player.mediaInfo.path) : t('workspace.noFile'),
);
const sidebarOpen = ref(false);
const sidebarToggle = ref<HTMLButtonElement>();
function toggleSidebar() {
  // Safari does not focus buttons on pointer clicks; preserve a return target.
  sidebarToggle.value?.focus();
  sidebarOpen.value = !sidebarOpen.value;
}
// The narrow playlist has a full-reader backdrop; unlike the permanent
// desktop sidebar it conceals both microphone controls and native preview.
useVideoMask(() => sidebarOpen.value && workspace.narrow);
const rail = ref<InstanceType<typeof WorkspaceRail> | null>(null);
const welcome = ref<InstanceType<typeof WorkspaceWelcome> | null>(null);
const welcomeAnnouncement = ref('');
function closeChat() {
  workspace.closeTool('chat');
  focusTool('chat');
}
function focusWelcome() {
  workspace.view = 'summary';
  workspace.chatOpen = false;
  workspace.playerOpen = false;
  welcomeAnnouncement.value = '';
  void nextTick(() => {
    welcome.value?.focusOpen();
    welcomeAnnouncement.value = t('chat.welcomeRequired');
  });
}
useWorkspaceLayout();
function focusTool(tool: WorkspaceTool) {
  void nextTick(() => rail.value?.focusTool(tool));
}
function closeTool(tool: WorkspaceTool, event: KeyboardEvent) {
  if (!toolMayClose(event)) return;
  workspace.closeTool(tool);
  focusTool(tool);
}
defineProps<{ videoElement?: HTMLElement | null }>();
</script>

<style scoped>
.summary-history { width: 100%; max-width: 760px; margin: 0 auto; padding: 16px 28px 28px; }
.summary-history button { display: flex; width: 100%; align-items: center; gap: 14px; padding: 18px 20px; text-align: left; border: 1px solid rgb(var(--velo-fg) / .10); border-radius: 16px; background: rgb(var(--velo-fg) / .025); color: rgb(var(--velo-fg) / .65); }
.summary-history button:hover { background: rgb(var(--velo-accent) / .05); }
.summary-history button:focus-visible { outline: 2px solid rgb(var(--velo-accent)); outline-offset: 3px; }
.history-copy { display: flex; flex: 1; flex-direction: column; gap: 4px; font-size: 12px; line-height: 1.7; }
.history-copy strong { font-size: 14px; font-weight: 600; color: rgb(var(--velo-fg) / .9); }
.history-action { font-size: 12px; white-space: nowrap; color: rgb(var(--velo-accent)); }
@media (max-width: 700px) { .summary-history { padding: 12px 16px 20px; } .history-action { display: none; } }
.sidebar-toggle,
.sidebar-backdrop {
  display: none;
}
.compact-view-title {
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
  .workspace-heading,
  .workspace-file-title {
    display: none;
  }
  .compact-view-title {
    display: block;
  }
}
</style>
