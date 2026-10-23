<template>
  <div
    ref="scrollElement"
    data-testid="summary-reader"
    class="summary-reader flex-1 min-h-0 overflow-y-auto overscroll-contain"
  >
    <div class="summary-stack min-h-full flex flex-col">
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
        v-else-if="
          !store.hasSummary && store.status && !store.status.configured
        "
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
          <p class="text-[10.5px] text-fg/30 mb-1">
            {{ t('summary.pullHint') }}
          </p>
          <button
            class="w-full px-2.5 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] font-mono text-[11px] text-fg/70 text-left transition-colors"
            @click="copy(PULL_COMMAND)"
          >
            {{ PULL_COMMAND }}
          </button>
        </div>
      </div>

      <!-- Running -->
      <div
        v-else-if="store.isRunning"
        class="flex-1 flex flex-col min-h-0"
      >
        <SummaryProgress
          :label="stageLabel"
          :stage="store.stage"
          :progress="store.progress"
          :streamed="store.streamed"
          :format-label="t(`summary.format.${settings.settings.summary.format ?? 'meeting'}`)"
          @cancel="store.cancel()"
        />
      </div>

      <!-- Finished -->
      <template v-else-if="store.hasSummary">
        <div
          class="summary-toolbar shrink-0 px-5 py-4 border-b border-fg/10 bg-surface space-y-3"
          :class="{ 'summary-toolbar-reading': reading }"
        >
          <div class="summary-model-row flex items-center gap-2 min-w-0">
            <Sparkles
              v-if="!reading"
              class="w-4 h-4 text-accent shrink-0"
              aria-hidden="true"
            />
            <component
              :is="reading ? 'h1' : 'h2'"
              :class="
                reading
                  ? 'text-[40px] leading-[48px] font-semibold tracking-tight text-fg/95'
                  : 'text-base font-semibold text-fg/95'
              "
              >{{ t('summary.readingTitle') }}</component
            >
            <span
              class="ml-auto text-[11px] text-fg/45 font-mono truncate"
              :title="store.summary?.model"
            >
              {{ t(`summary.format.${savedFormat}`) }} ·
              <span v-if="savedFormat === 'meeting' && savedDetail === 'detailed'" data-testid="saved-summary-detail">{{ t('summary.detail.detailed') }} ·</span>
              {{ store.summary?.model }}
            </span>
          </div>
          <div class="flex flex-wrap gap-2 summary-actions">
            <button
              @click="
                copy(
                  summaryForCopy(store.summary?.markdown ?? '', showTimestamps),
                )
              "
            >
              <Check v-if="copied" class="w-3.5 h-3.5 text-success" />
              <Copy v-else class="w-3.5 h-3.5" />
              <span aria-live="polite">{{
                copied ? t('summary.copied') : t('summary.copy')
              }}</span>
            </button>
            <button :disabled="chat.isRunning" @click="regenerate">
              <RefreshCw class="w-3.5 h-3.5" />
              {{ t('summary.again') }}
            </button>
            <button
              v-if="!reading"
              :aria-pressed="expanded"
              @click="emit('toggle-reading')"
            >
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
              <Check
                v-if="showTimestamps"
                class="w-3 h-3 text-accent"
                aria-hidden="true"
              />
            </button>
            <button
              @click="promptOpen = true"
              @keydown.space.stop.prevent="promptOpen = true"
            >
              <Pencil class="w-3.5 h-3.5" aria-hidden="true" />
              {{ t('summary.editPrompt') }}
              ·
              {{
                t(
                  `summary.format.${settings.settings.summary.format ?? 'meeting'}`,
                )
              }}
            </button>
          </div>
          <p v-if="optionsError" role="alert" class="text-xs text-danger">
            {{ optionsError }}
          </p>
          <div
            v-if="store.error"
            role="alert"
            class="rounded-xl border border-warning/25 bg-warning/10 px-4 py-3 text-sm leading-relaxed"
          >
            <p class="font-semibold text-fg/90">{{ t('summary.previousShown') }}</p>
            <p class="mt-1 text-fg/65">{{ displayError }}</p>
          </div>
        </div>

        <p
          v-if="(store.summary?.omitted_uncertain_claims ?? 0) > 0"
          data-testid="summary-omission"
          role="status"
          class="px-5 py-3 text-sm leading-relaxed text-fg/70 bg-warning/10 border-b border-warning/20"
        >
          {{ t('summary.omission', { count: store.summary?.omitted_uncertain_claims ?? 0 }) }}
        </p>
        <p
          v-if="store.isStale"
          class="px-3 py-2 text-[11px] text-amber-500/90 bg-amber-500/10 border-b border-amber-500/20"
        >
          {{ t('summary.stale') }}
        </p>

        <div
          class="summary-scroll shrink-0 px-4 py-5"
          :class="reading ? 'summary-reading' : 'bg-inset/[0.08]'"
        >
          <div
            class="mx-auto"
            :class="
              reading ? 'max-w-[64rem] space-y-7' : 'max-w-[46rem] space-y-4'
            "
          >
            <template
              v-for="(section, index) in sections"
              :key="`${store.summary?.created_at}-${index}`"
            >
              <details
                v-if="
                  savedFormat === 'meeting' &&
                  section.kind === 'topics' &&
                  sections.some((item) => item.kind !== 'topics' && item.nodes.length > 0)
                "
                class="summary-card group"
                :open="savedDetail === 'detailed'"
              >
                <summary
                  class="flex items-center gap-2 p-4 cursor-pointer list-none text-sm font-semibold text-fg/85 focus-visible:outline focus-visible:outline-accent rounded-2xl"
                >
                  <List
                    class="w-4 h-4 text-fg/45 shrink-0"
                    aria-hidden="true"
                  />
                  <span class="flex-1">{{ section.title }}</span>
                  <span
                    class="text-[11px] font-normal text-fg/45 group-open:hidden"
                    >{{ t('summary.showDetails') }}</span
                  >
                  <ChevronDown
                    class="w-4 h-4 text-fg/45 group-open:rotate-180"
                    aria-hidden="true"
                  />
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
                :class="{
                  'border-accent/20 bg-accent/[0.04]':
                    section.kind === 'overview',
                }"
              >
                <h3
                  v-if="section.title"
                  class="flex items-center gap-2 font-semibold text-fg/95 mb-3"
                  :class="reading ? 'text-[22px]' : 'text-sm'"
                >
                  <component
                    :is="sectionIcon(section.kind)"
                    class="w-4 h-4 shrink-0"
                    :class="
                      section.kind === 'actions'
                        ? 'text-success'
                        : 'text-accent'
                    "
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
            <p
              v-if="showTimestamps"
              class="text-[11px] leading-relaxed text-fg/40 px-1"
            >
              {{ t('summary.citationHint') }}
            </p>
          </div>
        </div>
      </template>

      <!-- Nothing yet: the one-click run lives here -->
      <div
        v-else
        class="summary-empty flex-1 min-h-0 overflow-y-auto flex flex-col items-center justify-center text-center p-8 gap-4"
      >
        <Sparkles class="w-8 h-8 text-fg/25" />
        <p class="text-xs text-fg/45 max-w-[15rem]">
          {{
            transcripts.hasTranscript
              ? t('summary.empty')
              : t('summary.emptyChained')
          }}
        </p>
        <button
          class="text-xs text-accent hover:underline"
          @click="promptOpen = true"
        >
          {{ t('summary.editPrompt') }}
          ·
          {{
            t(`summary.format.${settings.settings.summary.format ?? 'meeting'}`)
          }}
        </button>

        <button
          class="w-full h-9 rounded-xl bg-blue-500 hover:bg-blue-400 active:scale-[0.98] text-white text-[13px] font-semibold transition-all"
          :disabled="chat.isRunning"
          @click="!chat.isRunning && store.transcribeAndSummarise()"
        >
          {{
            transcripts.hasTranscript
              ? t('summary.runSummaryOnly')
              : t('summary.run')
          }}
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

        <div
          v-if="store.error"
          class="w-full text-[11px] text-danger/90 break-words"
        >
          <p>{{ displayError }}</p>
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
      <slot :scroll-element="scrollElement" />
    </div>
  </div>
  <SummaryPromptEditor :open="promptOpen" @close="promptOpen = false" />
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { useChatStore } from '@/stores/chatStore';
import {
  Check,
  Copy,
  FileAudio,
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
import SummaryProgress from './SummaryProgress.vue';
import { useSummaryStore } from '@/stores/summaryStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useI18n } from '@/composables/useI18n';
import {
  groupSummarySections,
  summaryForCopy,
  type SummarySection,
} from '@/utils/summaryMarkdown';
import { providerHost as hostFor } from '@/utils/providerLocation';

defineProps<{ expanded?: boolean; reading?: boolean }>();
const emit = defineEmits<{
  (e: 'goto-transcript'): void;
  (e: 'toggle-reading'): void;
}>();

const store = useSummaryStore();
const chat = useChatStore();
const scrollElement = ref<HTMLElement | null>(null);
watch(() => store.isRunning, async (running) => {
  if (!running) return;
  await nextTick();
  if (scrollElement.value) scrollElement.value.scrollTop = 0;
});
const transcripts = useTranscriptStore();
const settings = useSettingsStore();
const player = usePlayerStore();
const { t } = useI18n();

const PULL_COMMAND = 'ollama pull qwen3:30b-instruct';

const copied = ref(false);
const promptOpen = ref(false);
const optionsError = ref('');
const showTimestamps = computed(
  () => settings.settings.general.show_summary_timestamps === true,
);
async function toggleTimestamps() {
  if (settings.summaryOptionsSaving) return;
  optionsError.value = '';
  if (
    !(await settings.saveSummaryOptions({
      showTimestamps: !showTimestamps.value,
    }))
  )
    optionsError.value = t('summary.optionsSaveError');
}
const hasMedia = computed(() => player.mediaInfo !== null);
const isOllama = computed(
  () =>
    (store.status?.provider ?? settings.settings.summary.provider) === 'ollama',
);
const displayError = computed(() => {
  const error = store.error ?? '';
  const boundedErrors = {
    summary_work_limit: 'summary.workLimit',
    summary_review_unstable: 'summary.reviewUnstable',
    summary_review_invalid: 'summary.reviewInvalid',
    summary_layout_unsupported: 'summary.layoutUnsupportedParts',
    summary_part_layout_invalid: 'summary.partLayoutInvalid',
    summary_no_supported_content: 'summary.noSupportedContent',
  } as const;
  if (Object.hasOwn(boundedErrors, error))
    return t(boundedErrors[error as keyof typeof boundedErrors]);
  if (error === 'unsupported_summary_layout') return t('summary.unsupportedLayout');
  if (error === 'summary_output_limit' || error.includes('reached the output limit'))
    return t('summary.outputLimit');
  if (error === 'summary_context_limit' || error.includes('context budget'))
    return t('summary.contextLimit');
  if (error.includes('ended before completion')) return t('summary.incomplete');
  return error;
});
const savedFormat = computed(() => store.summary?.format ?? 'meeting');
const savedDetail = computed(() => store.summary?.meeting_detail ?? 'standard');
const sections = computed(() =>
  groupSummarySections(
    summaryForCopy(store.summary?.markdown ?? '', showTimestamps.value), savedFormat.value,
    transcripts.transcript?.path === store.summary?.path
      ? transcripts.transcript?.segments ?? [] : [],
    savedDetail.value,
  ),
);
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
  const total = Math.max(1, store.mapTotal);
  const done = Math.min(total, Math.max(1, store.mapped + 1));
  switch (store.stage) {
    case 'writing_parts':
      return t('summary.writingParts', { done, total });
    case 'checking_consistency':
      return t('summary.checkingConsistency', { done, total });
    case 'replacing_parts':
      return t('summary.replacingParts');
    case 'assembling_summary':
      return t('summary.assemblingParts');
    case 'reducing':
      return t('summary.reducing', { done: store.mapped + 1, total: store.mapTotal });
    case 'writing_summary':
      return t('summary.writingDirect');
    case 'checking_references':
      return t('summary.checkingReferences');
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
    case 'reviewing':
      return t('summary.reviewing');
    case 'extracting_evidence':
      return t('summary.extractingEvidence');
    case 'checking_evidence':
      return t('summary.checkingEvidence');
    case 'composing_summary':
      return t('summary.composingSummary');
    default:
      return t('summary.summarising');
  }
});

/// The two failures worth spelling out, since both have a one-line fix.
const errorHint = computed(() => {
  const message = store.error ?? '';
  if (message.includes('Could not reach')) {
    return t(
      isOllama.value
        ? 'summary.hintUnreachable'
        : 'summary.hintRemoteUnreachable',
    );
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

/// Keep the last result until a checked replacement arrives. The transcript
/// is never redone for a second opinion.
async function regenerate() {
  if (chat.isRunning) return;
  await store.summarise();
}

function transcribeOnly() {
  emit('goto-transcript');
  void transcripts.generate();
}
</script>

<style scoped>
.summary-toolbar-reading {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
  padding: 24px 40px 16px;
}
.summary-toolbar-reading .summary-model-row {
  display: contents;
}
.summary-toolbar-reading .summary-actions {
  margin: 0;
}
.summary-reading {
  padding: 32px 40px;
}
.summary-reading .summary-card {
  border: 0;
  background: transparent;
  border-radius: 0;
  padding: 0;
}
.summary-reading :deep(.summary-bullet) {
  width: 7px;
  height: 7px;
  background: rgb(var(--velo-accent));
}
.summary-reading details > summary {
  padding: 12px 0;
}
.summary-reading details > div {
  padding: 0 0 12px;
}
.summary-empty > button {
  max-width: 360px;
}
@media (max-height: 550px) {
  .summary-toolbar {
    padding: 8px 16px;
  }
  .summary-model-row,
  .summary-toolbar-reading .summary-model-row {
    display: none;
  }
  .summary-toolbar-reading {
    padding: 8px 16px;
  }
  .summary-reading {
    padding: 16px 24px;
  }
  .summary-actions {
    flex-wrap: nowrap;
    overflow-x: auto;
    margin-top: 0;
  }
  .summary-actions button {
    flex-shrink: 0;
  }
  .summary-scroll {
    min-height: 160px;
  }
  .summary-empty {
    justify-content: flex-start;
    padding: 12px 24px;
    gap: 8px;
  }
}
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
