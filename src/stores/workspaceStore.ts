import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type {
  WorkspaceMode,
  WorkspaceTool,
  WorkspaceView,
} from '@/types/workspace';

export const useWorkspaceStore = defineStore('workspace', () => {
  const view = ref<WorkspaceView>('summary');
  const mode = ref<WorkspaceMode>('reading');
  const chatOpen = ref(false);
  const playerOpen = ref(false);
  const viewportError = ref(false);
  const compact = ref(false);
  const chatVisible = computed(
    () =>
      view.value === 'summary' &&
      chatOpen.value &&
      mode.value === 'reading' &&
      !(compact.value && playerOpen.value),
  );

  function toggleTranscript() {
    view.value = view.value === 'summary' ? 'transcript' : 'summary';
  }
  function toggleChat() {
    if (view.value === 'transcript' || !chatVisible.value) {
      view.value = 'summary';
      chatOpen.value = true;
      if (compact.value) playerOpen.value = false;
    } else chatOpen.value = false;
  }
  function togglePlayer() {
    playerOpen.value = !playerOpen.value;
  }
  function showVideo() {
    mode.value = 'video';
  }
  function showReading() {
    mode.value = 'reading';
  }
  function resetForMedia() {
    view.value = 'summary';
    mode.value = 'reading';
    chatOpen.value = false;
    playerOpen.value = false;
    viewportError.value = false;
  }
  function closeTool(tool: WorkspaceTool) {
    if (tool === 'transcript') view.value = 'summary';
    if (tool === 'chat') chatOpen.value = false;
    if (tool === 'player') playerOpen.value = false;
  }
  return {
    view,
    mode,
    chatOpen,
    playerOpen,
    viewportError,
    compact,
    chatVisible,
    toggleTranscript,
    toggleChat,
    togglePlayer,
    closeTool,
    resetForMedia,
    showVideo,
    showReading,
  };
});
