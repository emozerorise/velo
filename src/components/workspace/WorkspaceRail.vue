<template>
  <nav class="workspace-rail shrink-0 w-16 border-l border-fg/10 bg-surface flex flex-col items-center gap-3 py-5" :aria-label="t('workspace.tools')">
    <button
      v-for="tool in tools"
      :key="tool.id"
      :data-testid="`tool-${tool.id}`"
      :aria-label="t(tool.label)"
      :title="t(tool.label)"
      :aria-pressed="tool.active"
      :aria-controls="tool.controls"
      class="w-11 h-11 grid place-items-center rounded-xl transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
      :class="tool.active ? 'text-accent bg-accent/10' : 'text-fg/45 hover:text-fg hover:bg-fg/5'"
      @click="tool.action()"
      @keydown.space.stop.prevent="tool.action()"
    >
      <component :is="tool.icon" class="w-[22px] h-[22px]" aria-hidden="true" />
    </button>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { FileText, MessageCircle, MonitorPlay } from '@lucide/vue';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { useI18n } from '@/composables/useI18n';
const workspace = useWorkspaceStore();
const { t } = useI18n();
const tools = computed(() => [
  { id: 'transcript', label: 'transcript.title', icon: FileText, active: workspace.view === 'transcript', controls: 'workspace-transcript', action: workspace.toggleTranscript },
  { id: 'chat', label: 'workspace.chatTitle', icon: MessageCircle, active: workspace.chatVisible, controls: 'workspace-chat', action: workspace.toggleChat },
  { id: 'player', label: 'workspace.playerTitle', icon: MonitorPlay, active: workspace.playerOpen, controls: 'workspace-player', action: workspace.togglePlayer },
] as const);
</script>
