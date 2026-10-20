<template>
  <div data-testid="workspace-welcome" class="welcome flex-1 min-h-0 overflow-y-auto px-6 py-8 text-center">
    <div class="max-w-[42rem] mx-auto flex flex-col items-center gap-5">
      <div class="welcome-mark w-24 h-24 rounded-full grid place-items-center text-accent border border-accent/20 bg-accent/[0.07]">
        <FileAudio class="w-11 h-11" :stroke-width="1.3" aria-hidden="true" />
      </div>
      <h2 class="text-2xl sm:text-[28px] font-semibold tracking-tight text-fg/95 leading-snug">{{ t('workspace.welcomeTitle') }}</h2>
      <p class="max-w-lg text-sm sm:text-base text-fg/60 leading-relaxed">{{ t('workspace.welcomeDescription') }}</p>
      <button data-testid="welcome-open-file" class="inline-flex items-center gap-3 rounded-2xl bg-blue-500 hover:bg-blue-400 px-6 py-3.5 text-white font-semibold text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-4" @click="openFileDialog">
        <CirclePlus class="w-5 h-5" aria-hidden="true" />{{ t('app.openMedia') }}
      </button>
      <p class="text-xs text-fg/40 leading-7">{{ t('workspace.dropHint') }}<br />MP3 · MP4 · WAV · {{ t('workspace.moreFormats') }}</p>
      <div class="welcome-steps grid grid-cols-3 gap-4 w-full mt-3">
        <div v-for="step in steps" :key="step.title" class="space-y-2">
          <component :is="step.icon" class="w-6 h-6 mx-auto text-fg/60" aria-hidden="true" />
          <p class="text-sm font-medium text-fg/85">{{ t(step.title) }}</p>
          <p class="text-xs text-fg/45 leading-relaxed">{{ t(step.description) }}</p>
        </div>
      </div>
      <p class="mt-3 text-xs text-fg/35">{{ t('workspace.manualOnly') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FileAudio, FileText, AudioLines, CirclePlus } from '@lucide/vue';
import { useI18n } from '@/composables/useI18n';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
const { t } = useI18n();
const { openFileDialog } = useKeyboardShortcuts();
const steps = [
  { icon: FileAudio, title: 'workspace.openFile', description: 'workspace.stepOpen' },
  { icon: AudioLines, title: 'workspace.stepTranscribeTitle', description: 'workspace.stepTranscribe' },
  { icon: FileText, title: 'workspace.stepSummaryTitle', description: 'workspace.stepSummary' },
] as const;
</script>

<style scoped>
.welcome { display: grid; align-items: center; }
@media (max-height: 650px) {
  .welcome { display: block; padding-top: 20px; }
  .welcome-mark, .welcome-steps { display: none; }
}
</style>
