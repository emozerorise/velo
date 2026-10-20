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

    <div v-if="isOllama" class="w-full text-left">
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
    <div class="shrink-0 px-5 py-4 border-b border-fg/10 bg-surface space-y-3">
      <div class="flex items-center gap-2 min-w-0">
        <Sparkles class="w-4 h-4 text-accent shrink-0" aria-hidden="true" />
        <h2 class="text-base font-semibold text-fg/95">{{ t('summary.readingTitle') }}</h2>
        <span
          class="ml-auto text-[11px] text-fg/45 font-mono truncate"
          :title="store.summary?.model"
        >
          {{ store.summary?.model }}
        </span>
      </div>
      <div class="flex flex-wrap gap-2 summary-actions">
        <button @click="copy(summaryForCopy(store.summary?.markdown ?? '', showTimestamps))">
          <Check v-if="copied" class="w-3.5 h-3.5 text-success" />
          <Copy v-else class="w-3.5 h-3.5" />
          <span aria-live="polite">{{ copied ? t('summary.copied') : t('summary.copy') }}</span>
        </button>
        <button @click="regenerate">
          <RefreshCw class="w-3.5 h-3.5" />
          {{ t('summary.again') }}
        </button>
        <button v-if="!reading" :aria-pressed="expanded" @click="emit('toggle-reading')">
          <Minimize2 v-if="expanded" class="w-3.5 h-3.5" />
          <Maximize2 v-else class="w-3.5 h-3.5" />
          {{ expanded ? t('summary.collapse') : t('summary.expand') }}
        </button>
        <button
          :aria-pressed="showTimestamps"
          :disabled="settings.summaryOptionsSaving"
          @click="toggleTimestamps"
          @keydown.space.stop.prevent="toggleTimestamps"
        >
          <Clock class="w-3.5 h-3.5" aria-hidden="true" />
          {{ t('summary.showTimestamps') }}
          <Check v-if="showTimestamps" class="w-3 h-3 text-accent" aria-hidden="true" />
        </button>
        <button @click="promptOpen = true" @keydown.space.stop.prevent="promptOpen = true">
          <Pencil class="w-3.5 h-3.5" aria-hidden="true" />
          {{ t('summary.editPrompt') }}
        </button>
      </div>
      <p v-if="optionsError" role="alert" class="text-xs text-danger">{{ optionsError }}</p>
    </div>

    <p
      v-if="store.isStale"
      class="px-3 py-2 text-[11px] text-amber-500/90 bg-amber-500/10 border-b border-amber-500/20"
    >
      {{ t('summary.stale') }}
    </p>

    <div class="flex-1 min-h-0 overflow-y-auto overscroll-contain px-4 py-5 bg-inset/[0.08]">
      <div class="max-w-[46rem] mx-auto space-y-4">
        <template
          v-for="(section, index) in sections"
          :key="`${store.summary?.created_at}-${index}`"
        >
          <details v-if="section.kind === 'topics'" class="summary-card group">
            <summary
              class="flex items-center gap-2 p-4 cursor-pointer list-none text-sm font-semibold text-fg/85 focus-visible:outline focus-visible:outline-accent rounded-2xl"
            >
              <List class="w-4 h-4 text-fg/45 shrink-0" aria-hidden="true" />
              <span class="flex-1">{{ section.title }}</span>
              <span class="text-[11px] font-normal text-fg/45 group-open:hidden">{{
                t('summary.showDetails')
              }}</span>
              <ChevronDown class="w-4 h-4 text-fg/45 group-open:rotate-180" aria-hidden="true" />
            </summary>
            <div class="px-4 pb-4">
              <SummarySectionBody
                :nodes="section.nodes"
                :show-timestamps="showTimestamps"
                @seek="store.jumpTo"
              />
            </div>
          </details>
          <section
            v-else
            class="summary-card p-4"
            :class="{ 'border-accent/20 bg-accent/[0.04]': section.kind === 'overview' }"
          >
            <h3
              v-if="section.title"
              class="flex items-center gap-2 text-sm font-semibold text-fg/95 mb-3"
            >
              <component
                :is="sectionIcon(section.kind)"
                class="w-4 h-4 shrink-0"
                :class="section.kind === 'actions' ? 'text-success' : 'text-accent'"
                aria-hidden="true"
              />
              {{ section.title }}
            </h3>
            <SummarySectionBody
              :nodes="section.nodes"
              :show-timestamps="showTimestamps"
              @seek="store.jumpTo"
            />
          </section>
        </template>
        <p v-if="showTimestamps" class="text-[11px] leading-relaxed text-fg/40 px-1">
          {{ t('summary.citationHint') }}
        </p>
      </div>
    </div>
  </template>

  <!-- Nothing yet: the one-click run lives here -->
  <div v-else class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-4">
    <Sparkles class="w-8 h-8 text-fg/25" />
    <p class="text-xs text-fg/45 max-w-[15rem]">
      {{ transcripts.hasTranscript ? t('summary.empty') : t('summary.emptyChained') }}
    </p>
    <button class="text-xs text-accent hover:underline" @click="promptOpen = true">
      {{ t('summary.editPrompt') }}
    </button>

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
  <SummaryPromptEditor :open="promptOpen" @close="promptOpen = false" />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  Check,
  Copy,
  FileAudio,
  Loader2,
  RefreshCw,
  Sparkles,
  Maximize2,
  Minimize2,
  List,
  ChevronDown,
  CircleCheck,
  ListTodo,
  CircleHelp,
  AlignLeft,
  Clock,
  Pencil,
} from '@lucide/vue';
import SummarySectionBody from './SummarySectionBody.vue';
import SummaryPromptEditor from './SummaryPromptEditor.vue';
import { useSummaryStore } from '@/stores/summaryStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useI18n } from '@/composables/useI18n';
import { groupSummarySections, summaryForCopy, type SummarySection } from '@/utils/summaryMarkdown';
import { providerHost as hostFor } from '@/utils/providerLocation';

defineProps<{ expanded?: boolean; reading?: boolean }>();
const emit = defineEmits<{ (e: 'goto-transcript'): void; (e: 'toggle-reading'): void }>();

const store = useSummaryStore();
const transcripts = useTranscriptStore();
const settings = useSettingsStore();
const player = usePlayerStore();
const { t } = useI18n();

const PULL_COMMAND = 'ollama pull qwen3:30b-instruct';

const copied = ref(false);
const promptOpen = ref(false);
const optionsError = ref('');
const showTimestamps = computed(() => settings.settings.general.show_summary_timestamps !== false);
async function toggleTimestamps() {
  if (settings.summaryOptionsSaving) return;
  optionsError.value = '';
  if (!(await settings.saveSummaryOptions({ showTimestamps: !showTimestamps.value })))
    optionsError.value = t('summary.optionsSaveError');
}
const hasMedia = computed(() => player.mediaInfo !== null);
const isOllama = computed(
  () => (store.status?.provider ?? settings.settings.summary.provider) === 'ollama',
);
const sections = computed(() => groupSummarySections(store.summary?.markdown ?? ''));
function sectionIcon(kind: SummarySection['kind']) {
  return {
    overview: AlignLeft,
    decisions: CircleCheck,
    actions: ListTodo,
    topics: List,
    questions: CircleHelp,
    other: AlignLeft,
  }[kind];
}

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
  if (message.includes('Could not reach')) {
    return t(isOllama.value ? 'summary.hintUnreachable' : 'summary.hintRemoteUnreachable');
  }
  if (message.includes('took too long')) return t('summary.hintSlow');
  if (message.includes('not available on this server')) {
    return t(isOllama.value ? 'summary.hintModel' : 'summary.hintRemoteModel', {
      model: store.status?.model ?? '',
    });
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

<style scoped>
.summary-actions button {
  @apply inline-flex items-center gap-1.5 rounded-lg border border-fg/10 bg-fg/[0.03] px-2.5 py-2 text-xs text-fg/70 transition-colors hover:bg-fg/[0.08] hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent;
}
.summary-card {
  @apply rounded-2xl border border-fg/[0.08] bg-surface;
}
summary::-webkit-details-marker {
  display: none;
}
</style>
