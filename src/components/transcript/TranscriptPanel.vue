<template>
  <Transition name="slide">
    <div
      v-if="store.isPanelOpen"
      class="fixed inset-y-0 right-0 z-40 w-[26rem] max-w-full bg-surface border-l border-fg/10 flex flex-col pointer-events-auto"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3.5 border-b border-fg/10">
        <div class="flex items-center gap-2 min-w-0">
          <Captions class="w-4 h-4 text-accent shrink-0" />
          <span class="text-sm font-medium text-fg/90">{{ t('transcript.title') }}</span>
          <span v-if="store.transcript" class="text-xs text-fg/40 font-mono truncate">
            {{ store.transcript.language }} · {{ store.transcript.segments.length }}
          </span>
        </div>

        <button
          class="p-1 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
          @click="store.isPanelOpen = false"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Tabs. Reading the transcript and reading the summary are different
           jobs, and both want the whole drawer. -->
      <div class="flex items-center gap-1 px-3 pt-2 pb-1.5 border-b border-fg/10">
        <button
          v-for="option in TABS"
          :key="option.id"
          :class="[
            'px-2.5 py-1 rounded-lg text-[11.5px] font-medium transition-colors',
            tab === option.id
              ? 'bg-blue-600/25 text-accent'
              : 'text-fg/50 hover:text-fg hover:bg-fg/5',
          ]"
          @click="tab = option.id"
        >
          {{ t(option.key) }}
        </button>
      </div>

      <TranscriptTab v-if="tab === 'transcript'" />
      <SummaryTab v-else @goto-transcript="tab = 'transcript'" />
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Captions, X } from '@lucide/vue';
import TranscriptTab from './TranscriptTab.vue';
import SummaryTab from './SummaryTab.vue';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useI18n } from '@/composables/useI18n';

const store = useTranscriptStore();
const { t } = useI18n();

const TABS = [
  { id: 'transcript', key: 'summary.tab.transcript' },
  { id: 'summary', key: 'summary.tab.summary' },
] as const;

// App.vue keeps this panel mounted, so the chosen tab survives the drawer
// being closed and reopened.
const tab = ref<(typeof TABS)[number]['id']>('transcript');
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
