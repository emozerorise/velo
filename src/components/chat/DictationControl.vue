<template>
  <div class="contents" @keydown.stop="onKey">
    <button
      type="button"
      data-testid="dictation-toggle"
      :aria-label="t(recording ? 'dictation.stop' : 'dictation.start')"
      :disabled="!recording && !dictation.canStart"
      :class="recording ? 'border-danger text-danger bg-danger/10' : 'border-fg/20 text-fg/75 hover:bg-fg/10'"
      class="w-11 h-11 shrink-0 rounded-full border grid place-items-center disabled:opacity-40 focus-visible:outline focus-visible:outline-accent focus-visible:outline-offset-2"
      @click="recording ? dictation.stop() : dictation.start()"
    >
      <Square v-if="recording" class="w-4 h-4 fill-current" aria-hidden="true" />
      <Mic v-else class="w-5 h-5" aria-hidden="true" />
    </button>
    <div v-if="dictation.active" class="flex flex-wrap items-center gap-2 text-sm">
      <span role="status" class="text-fg/80">{{ t(phaseLabel) }}</span>
      <span v-if="recording" role="timer" aria-live="off" class="font-mono text-danger">{{ elapsed }} / 1:00</span>
      <button type="button" data-testid="dictation-cancel" class="p-2 rounded-lg text-fg/75 underline focus-visible:outline focus-visible:outline-accent" @click="cancel">{{ t('dictation.cancel') }}</button>
    </div>
    <p v-if="dictation.pausedPlayback && dictation.active" class="basis-full order-last text-xs text-fg/65">{{ t('dictation.paused') }}</p>
    <section v-if="dictation.phase === 'review'" class="basis-full order-last rounded-2xl border border-accent/25 bg-accent/[0.04] p-3 space-y-3" :aria-label="t('dictation.review')">
      <label for="dictation-proposal" class="block text-sm font-medium text-fg/90">{{ t('dictation.review') }}</label>
      <textarea id="dictation-proposal" v-model="dictation.proposal" data-testid="dictation-proposal" rows="3" class="block w-full max-h-40 resize-y rounded-lg border border-fg/20 bg-transparent p-2 text-base leading-relaxed text-fg/90 focus-visible:outline focus-visible:outline-accent" />
      <p class="text-xs leading-relaxed text-fg/65">{{ t('dictation.reviewHint') }}</p>
      <div class="flex flex-wrap gap-2">
        <button type="button" data-testid="dictation-insert" :disabled="!dictation.proposal?.trim()" class="rounded-lg bg-blue-600 px-4 py-2 text-sm text-white disabled:opacity-40 focus-visible:outline focus-visible:outline-accent focus-visible:outline-offset-2" @click="insert">{{ t('dictation.insert') }}</button>
        <button type="button" data-testid="dictation-discard" class="rounded-lg border border-fg/20 px-4 py-2 text-sm text-fg/80 hover:bg-fg/5 focus-visible:outline focus-visible:outline-accent" @click="discard">{{ t('dictation.discard') }}</button>
      </div>
    </section>
    <div v-if="dictation.errorCode" class="basis-full order-last text-sm leading-relaxed text-fg/80">
      <p role="alert">{{ t(`dictation.error.${dictation.errorCode}`) }}</p>
      <button v-if="dictation.errorCode === 'missing_model'" type="button" class="mt-1 text-accent underline" @click="settings.openSettings('transcript')">{{ t('dictation.setup') }}</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import { Mic, Square } from '@lucide/vue';
import { useDictationStore } from '@/stores/dictationStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
const emit = defineEmits<{ (event: 'focus'): void }>();
const dictation = useDictationStore();
const settings = useSettingsStore();
const { t } = useI18n();
const recording = computed(() => dictation.phase === 'recording');
const phaseLabel = computed(() => dictation.phase === 'recording' ? 'dictation.recording' : dictation.phase === 'transcribing' ? 'dictation.transcribing' : 'dictation.permission');
const elapsed = computed(() => { const seconds = Math.floor(dictation.elapsedMs / 1000); return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`; });
function insert() { if (dictation.insert()) emit('focus'); }
function discard() { dictation.discard(); emit('focus'); }
function cancel() { void dictation.cancel(); emit('focus'); }
function onKey(event: KeyboardEvent) { if (event.key === 'Escape' && (dictation.active || dictation.phase === 'review')) { event.preventDefault(); cancel(); } }
</script>
