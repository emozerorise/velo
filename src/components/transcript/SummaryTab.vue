<template>
  <!-- No media -->
  <div
    v-if="!hasMedia"
    class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-2 text-fg/40"
  >
    <FileAudio class="w-8 h-8 opacity-40" />
    <p class="text-xs">{{ t('summary.openFirst') }}</p>
  </div>

  <!-- Provider not set up -->
  <div
    v-else-if="store.status && !store.status.configured"
    class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-4"
  >
    <Sparkles class="w-8 h-8 text-fg/25" />
    <div>
      <p class="text-xs font-medium text-fg/80">
        {{ t('summary.notConfigured') }}
      </p>
      <p class="mt-1.5 text-[11.5px] leading-relaxed text-fg/45">
        {{ t('summary.notConfiguredDesc') }}
      </p>
    </div>

    <button
      class="w-full h-9 rounded-xl bg-blue-500 hover:bg-blue-400 active:scale-[0.98] text-white text-[13px] font-semibold transition-all"
      @click="settings.openSettings('ai')"
    >
      {{ t('summary.openSettings') }}
    </button>

    <div class="w-full text-left">
      <p class="text-[10.5px] text-fg/30 mb-1">{{ t('summary.pullHint') }}</p>
      <button
        class="w-full px-2.5 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] font-mono text-[11px] text-fg/70 text-left transition-colors"
        @click="copy(PULL_COMMAND)"
      >
        {{ PULL_COMMAND }}
      </button>
    </div>
  </div>

  <!-- Running -->
  <div v-else-if="store.isRunning" class="flex-1 flex flex-col min-h-0 p-5 gap-4">
    <div class="w-full">
      <div class="flex items-center justify-between text-[11px] text-fg/60 mb-1.5">
        <span class="flex items-center gap-1.5">
          <Loader2 class="w-3 h-3 animate-spin text-accent" />
          {{ stageLabel }}
        </span>
        <span v-if="store.chained" class="font-mono text-fg/35">{{ stepLabel }}</span>
      </div>
      <div class="h-1 rounded-full bg-fg/10 overflow-hidden">
        <div
          class="h-full bg-blue-500 transition-[width] duration-300"
          :class="{ 'animate-pulse w-1/3': store.progress < 0 }"
          :style="store.progress >= 0 ? { width: percentLabel } : undefined"
        />
      </div>
    </div>

    <!-- The final pass streams in, so a long answer is readable as it lands. -->
    <div
      v-if="store.streamed"
      class="flex-1 min-h-0 overflow-y-auto text-[12.5px] leading-relaxed text-fg/70 whitespace-pre-wrap"
    >
      {{ store.streamed }}
    </div>
    <p v-else class="flex-1 text-[11px] text-fg/35 text-center">
      {{ t('summary.runningNote') }}
    </p>

    <button
      class="self-center px-3 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] text-fg/80 text-xs transition-colors"
      @click="store.cancel()"
    >
      {{ t('summary.cancel') }}
    </button>
  </div>

  <!-- Finished -->
  <template v-else-if="store.hasSummary">
    <div class="flex items-center gap-2 px-3 py-2 border-b border-fg/5 bg-inset/20">
      <span class="flex-1 text-[11px] text-fg/40 font-mono truncate">
        {{ store.summary?.model }}
      </span>

      <button
        class="p-1.5 rounded text-fg/40 hover:text-fg transition-colors"
        :title="t('summary.copy')"
        @click="copy(store.summary?.markdown ?? '')"
      >
        <Check v-if="copied" class="w-3.5 h-3.5 text-success" />
        <Copy v-else class="w-3.5 h-3.5" />
      </button>

      <button
        class="p-1.5 rounded text-fg/40 hover:text-fg transition-colors"
        :title="t('summary.again')"
        @click="regenerate"
      >
        <RefreshCw class="w-3.5 h-3.5" />
      </button>
    </div>

    <p
      v-if="store.isStale"
      class="px-3 py-2 text-[11px] text-amber-500/90 bg-amber-500/10 border-b border-amber-500/20"
    >
      {{ t('summary.stale') }}
    </p>

    <div class="flex-1 overflow-y-auto px-4 py-3">
      <template v-for="(node, index) in nodes" :key="index">
        <h3
          v-if="node.type === 'heading'"
          class="text-[11px] font-semibold uppercase tracking-wide text-accent mt-4 first:mt-0 mb-1.5"
        >
          {{ node.text }}
        </h3>

        <p
          v-else-if="node.type === 'paragraph'"
          class="text-[12.5px] leading-relaxed text-fg/75 mb-2"
        >
          <SummaryLine :parts="node.parts" @seek="store.jumpTo" />
        </p>

        <div
          v-else
          class="flex gap-1.5 text-[12.5px] leading-relaxed text-fg/75 mb-1"
          :style="{ paddingLeft: `${node.depth * 0.9}rem` }"
        >
          <span class="text-fg/25 select-none">•</span>
          <span><SummaryLine :parts="node.parts" @seek="store.jumpTo" /></span>
        </div>
      </template>
    </div>
  </template>

  <!-- Nothing yet: the one-click run lives here -->
  <div v-else class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-4">
    <Sparkles class="w-8 h-8 text-fg/25" />
    <p class="text-xs text-fg/45 max-w-[15rem]">
      {{ transcripts.hasTranscript ? t('summary.empty') : t('summary.emptyChained') }}
    </p>

    <button
      class="w-full h-9 rounded-xl bg-blue-500 hover:bg-blue-400 active:scale-[0.98] text-white text-[13px] font-semibold transition-all"
      @click="store.transcribeAndSummarise()"
    >
      {{ transcripts.hasTranscript ? t('summary.runSummaryOnly') : t('summary.run') }}
    </button>

    <button
      v-if="!transcripts.hasTranscript"
      class="text-[11px] text-fg/45 hover:text-fg/80 transition-colors"
      @click="transcribeOnly"
    >
      {{ t('summary.transcriptOnly') }}
    </button>

    <p
      class="text-[10.5px] leading-relaxed"
      :class="store.status?.remote ? 'text-amber-500/80' : 'text-fg/30'"
    >
      {{
        store.status?.remote
          ? t('summary.remoteNote', { host: providerHost })
          : t('summary.localNote')
      }}
    </p>

    <div v-if="store.error" class="w-full text-[11px] text-danger/90 break-words">
      <p>{{ store.error }}</p>
      <p v-if="errorHint" class="mt-1 text-fg/45">{{ errorHint }}</p>
      <button
        v-if="store.apiKeyRejected"
        class="mt-1.5 text-accent hover:underline"
        @click="settings.openSettings('ai')"
      >
        {{ t('summary.fixApiKey') }}
      </button>

      <!-- Whatever arrived before it failed is still worth reading. -->
      <div
        v-if="store.streamed"
        class="mt-2 max-h-40 overflow-y-auto text-left text-fg/55 whitespace-pre-wrap"
      >
        {{ store.streamed }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Check, Copy, FileAudio, Loader2, RefreshCw, Sparkles } from '@lucide/vue';
import SummaryLine from './SummaryLine.vue';
import { useSummaryStore } from '@/stores/summaryStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useI18n } from '@/composables/useI18n';
import { parseSummary } from '@/utils/summaryMarkdown';
import { providerHost as hostFor } from '@/utils/providerLocation';

const emit = defineEmits<{ (e: 'goto-transcript'): void }>();

const store = useSummaryStore();
const transcripts = useTranscriptStore();
const settings = useSettingsStore();
const player = usePlayerStore();
const { t } = useI18n();

const PULL_COMMAND = 'ollama pull qwen3:8b';

const copied = ref(false);
const hasMedia = computed(() => player.mediaInfo !== null);
const nodes = computed(() => parseSummary(store.summary?.markdown ?? ''));

const providerHost = computed(() => {
  const url = store.status?.base_url ?? '';
  return hostFor(url);
});

const stageLabel = computed(() => {
  switch (store.stage) {
    case 'checking':
      return t('summary.checking');
    case 'extracting':
      return t('summary.extracting');
    case 'transcribing':
      return t('summary.transcribing');
    case 'mapping':
      // A transcript small enough for one pass never shows a section count.
      return store.mapTotal > 1
        ? t('summary.mapping', {
            done: store.mapped + 1,
            total: store.mapTotal,
          })
        : t('summary.summarising');
    default:
      return t('summary.summarising');
  }
});

const stepLabel = computed(() => t('summary.stepOf', { step: store.step }));
const percentLabel = computed(() => `${Math.round(Math.max(0, store.progress) * 100)}%`);

/// The two failures worth spelling out, since both have a one-line fix.
const errorHint = computed(() => {
  const message = store.error ?? '';
  if (message.includes('Could not reach')) return t('summary.hintUnreachable');
  if (message.includes('took too long')) return t('summary.hintSlow');
  if (message.includes('not available on this server')) {
    return t('summary.hintModel', { model: store.status?.model ?? '' });
  }
  return null;
});

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    window.setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    console.error('Failed to copy:', e);
  }
}

/// Discard, then run again -- which re-summarises only. The transcript is
/// never redone for a second opinion.
async function regenerate() {
  await store.discard();
  await store.summarise();
}

function transcribeOnly() {
  emit('goto-transcript');
  void transcripts.generate();
}
</script>
