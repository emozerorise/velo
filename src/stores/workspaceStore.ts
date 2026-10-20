import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { WorkspaceMode, WorkspaceView } from '@/types/workspace';

export const useWorkspaceStore = defineStore('workspace', () => {
  const view = ref<WorkspaceView>('summary');
  const mode = ref<WorkspaceMode>('reading');
  const chatOpen = ref(false);
  const playerOpen = ref(false);
  const chatVisible = computed(() => view.value === 'summary' && chatOpen.value && mode.value === 'reading');

  function toggleTranscript() { view.value = view.value === 'summary' ? 'transcript' : 'summary'; }
  function toggleChat() {
    if (view.value === 'transcript') { view.value = 'summary'; chatOpen.value = true; }
    else chatOpen.value = !chatOpen.value;
  }
  function togglePlayer() { playerOpen.value = !playerOpen.value; }
  function showVideo() { mode.value = 'video'; }
  function showReading() { mode.value = 'reading'; }
  function resetForMedia() {
    view.value = 'summary';
    mode.value = 'reading';
    chatOpen.value = false;
    playerOpen.value = false;
  }
  return { view, mode, chatOpen, playerOpen, chatVisible, toggleTranscript, toggleChat, togglePlayer, resetForMedia, showVideo, showReading };
});
