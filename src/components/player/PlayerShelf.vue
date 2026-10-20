<template>
  <section
    v-show="workspace.mode === 'reading' && workspace.playerOpen"
    id="workspace-player"
    data-testid="player-shelf"
    class="player-shelf shrink-0 border-t border-fg/10"
    :aria-label="t('workspace.playerTitle')"
    @keydown.space.stop
    @keydown.esc.stop="closeFromKey"
  >
    <header class="shelf-header bg-surface h-9 px-4 flex items-center gap-2">
      <MonitorPlay class="w-4 h-4 text-accent shrink-0" aria-hidden="true" />
      <span class="truncate text-xs text-fg/55">{{
        player.mediaInfo?.file_name ?? t('workspace.playerTitle')
      }}</span>
      <button
        v-if="hasVideo"
        data-testid="expand-player"
        class="ml-auto p-1.5 rounded-md text-fg/55 hover:bg-fg/5 hover:text-fg"
        :aria-label="t('workspace.watchVideo')"
        @click="transcripts.showVideo()"
      >
        <Maximize2 class="w-4 h-4" />
      </button>
      <button
        data-testid="close-player"
        class="p-1.5 rounded-md text-fg/55 hover:bg-fg/5 hover:text-fg"
        :class="{ 'ml-auto': !hasVideo }"
        :aria-label="t('workspace.closePlayer')"
        @click="close"
      >
        <X class="w-4 h-4" />
      </button>
    </header>
    <div v-if="player.mediaInfo" class="flex min-w-0">
      <div
        v-if="hasVideo"
        ref="preview"
        data-testid="video-preview"
        :data-masked="masked || workspace.viewportError"
        class="native-preview shrink-0 relative"
      >
        <div
          v-if="masked || workspace.viewportError"
          class="absolute inset-0 bg-surface"
        />
      </div>
      <div class="flex-1 min-w-0 flex flex-col justify-center bg-surface">
        <ReadingTransport compact />
      </div>
    </div>
    <div
      v-else
      class="p-4 text-sm bg-surface flex flex-wrap items-center gap-3"
    >
      <p class="text-fg/55">{{ t('summary.openFirst') }}</p>
      <button
        data-testid="empty-player-open-file"
        class="px-3 py-2 rounded-lg text-accent bg-accent/10 hover:bg-accent/20"
        @click="openFileDialog"
      >
        {{ t('workspace.openFile') }}
      </button>
    </div>
    <p
      v-if="workspace.viewportError"
      role="alert"
      class="p-3 text-xs text-danger bg-surface"
    >
      {{ t('workspace.viewportError') }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Maximize2, MonitorPlay, X } from '@lucide/vue';
import ReadingTransport from './ReadingTransport.vue';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useUpdateStore } from '@/stores/updateStore';
import { useVideoViewport } from '@/composables/useVideoViewport';
import { isVideoMasked } from '@/composables/useVideoMask';
import { useI18n } from '@/composables/useI18n';
import { toolMayClose } from '@/utils/toolKeyboard';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
const props = defineProps<{ videoElement?: HTMLElement | null }>();
const workspace = useWorkspaceStore();
const player = usePlayerStore();
const transcripts = useTranscriptStore();
const updates = useUpdateStore();
const { t } = useI18n();
const { openFileDialog } = useKeyboardShortcuts();
const emit = defineEmits<{ (event: 'close'): void }>();
function close() {
  workspace.closeTool('player');
  emit('close');
}
function closeFromKey(event: KeyboardEvent) {
  if (toolMayClose(event)) close();
}
const preview = ref<HTMLElement | null>(null);
const hasVideo = computed(
  () =>
    !!player.mediaInfo?.video_codec &&
    player.mediaInfo.width > 0 &&
    player.mediaInfo.height > 0,
);
const masked = computed(() => isVideoMasked.value || updates.blocking);
// One observer owns both rectangles, so a hidden shelf cannot send a late
// hide after the large-video observer has already opened its surface.
const activeElement = computed(() =>
  workspace.mode === 'video' ? (props.videoElement ?? null) : preview.value,
);
useVideoViewport(
  activeElement,
  () =>
    hasVideo.value &&
    !masked.value &&
    (workspace.mode === 'video' || workspace.playerOpen),
);
</script>

<style scoped>
.native-preview {
  width: 240px;
  height: 135px;
}
@media (max-width: 760px), (max-height: 550px) {
  .native-preview {
    width: 160px;
    height: 90px;
  }
  .shelf-header {
    height: 32px;
  }
}
</style>
