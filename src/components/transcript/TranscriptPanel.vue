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

      <!-- No media -->
      <div
        v-if="!hasMedia"
        class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-2 text-fg/40"
      >
        <FileAudio class="w-8 h-8 opacity-40" />
        <p class="text-xs">{{ t('transcript.openFirst') }}</p>
      </div>

      <!-- Model downloading -->
      <div
        v-else-if="store.isDownloading"
        class="flex-1 flex flex-col items-center justify-center p-8 gap-4"
      >
        <Loader2 class="w-6 h-6 text-accent animate-spin" />
        <div class="w-full">
          <div class="flex items-center justify-between text-[11px] text-fg/60 mb-1.5">
            <span>{{ t('transcript.downloading') }}</span>
            <span class="font-mono">{{ downloadLabel }}</span>
          </div>
          <div class="h-1 rounded-full bg-fg/10 overflow-hidden">
            <div
              class="h-full bg-blue-500 transition-[width] duration-300"
              :style="{ width: downloadPercent }"
            />
          </div>
        </div>
        <button
          class="px-3 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] text-fg/80 text-xs transition-colors"
          @click="store.cancelDownload()"
        >
          {{ t('transcript.cancel') }}
        </button>
      </div>

      <!-- Model not downloaded yet -->
      <div
        v-else-if="needsModel"
        class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-4"
      >
        <Download class="w-8 h-8 text-fg/25" />
        <div>
          <p class="text-xs font-medium text-fg/80">{{ t('transcript.modelMissing') }}</p>
          <p class="mt-1.5 text-[11.5px] leading-relaxed text-fg/45">
            {{ t('transcript.modelExplain', { size: modelSize }) }}
          </p>
          <p class="mt-1 text-[11px] text-fg/30">{{ t('transcript.modelPrivacy') }}</p>
        </div>

        <button
          class="w-full h-9 rounded-xl bg-blue-500 hover:bg-blue-400 active:scale-[0.98] text-white text-[13px] font-semibold transition-all"
          @click="store.downloadModel()"
        >
          {{ t('transcript.download') }}
        </button>

        <p v-if="store.modelError" class="text-[11px] text-danger/90 break-words">
          {{ store.modelError }}
        </p>
      </div>

      <!-- Running -->
      <div v-else-if="store.isRunning" class="flex-1 flex flex-col items-center justify-center p-8 gap-4">
        <Loader2 class="w-6 h-6 text-accent animate-spin" />
        <div class="w-full">
          <div class="flex items-center justify-between text-[11px] text-fg/60 mb-1.5">
            <span>{{ stageLabel }}</span>
            <span v-if="store.progress >= 0" class="font-mono">{{ percentLabel }}</span>
          </div>
          <div class="h-1 rounded-full bg-fg/10 overflow-hidden">
            <div
              class="h-full bg-blue-500 transition-[width] duration-300"
              :class="{ 'animate-pulse w-1/3': store.progress < 0 }"
              :style="store.progress >= 0 ? { width: percentLabel } : undefined"
            />
          </div>
        </div>
        <p class="text-[11px] text-fg/35 text-center">{{ t('transcript.runningNote') }}</p>
        <button
          class="px-3 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] text-fg/80 text-xs transition-colors"
          @click="store.cancel()"
        >
          {{ t('transcript.cancel') }}
        </button>
      </div>

      <!-- Empty: offer to generate -->
      <div
        v-else-if="!store.hasTranscript"
        class="flex-1 flex flex-col items-center justify-center text-center p-8 gap-4"
      >
        <FileText class="w-8 h-8 text-fg/25" />
        <p class="text-xs text-fg/45 max-w-[15rem]">{{ t('transcript.empty') }}</p>

        <div class="w-full text-left">
          <label for="transcript-vocabulary" class="block text-[11px] text-fg/50 mb-1.5">
            {{ t('transcript.vocabulary') }}
            <span class="text-fg/25">{{ t('transcript.optional') }}</span>
          </label>
          <textarea
            id="transcript-vocabulary"
            v-model="store.prompt"
            rows="3"
            :placeholder="t('transcript.vocabularyPlaceholder')"
            class="w-full px-2.5 py-2 rounded-lg bg-fg/[0.07] text-fg/85 text-[11.5px] leading-relaxed outline-none placeholder:text-fg/20 resize-none"
          />
          <p class="mt-1.5 text-[10.5px] text-fg/30 leading-relaxed">
            {{ t('transcript.vocabularyHint') }}
          </p>
        </div>

        <label class="w-full flex items-center justify-between gap-2 text-[11px] text-fg/50">
          <span>{{ t('transcript.spokenLanguage') }}</span>
          <select
            v-model="store.language"
            class="flex-1 max-w-[9rem] px-2 py-1.5 rounded-lg bg-fg/[0.07] text-fg/85 text-xs outline-none"
          >
            <option v-for="lang in TRANSCRIPT_LANGUAGES" :key="lang.code" :value="lang.code">
              {{ t(lang.key) }}
            </option>
          </select>
        </label>

        <button
          class="w-full h-9 rounded-xl bg-blue-500 hover:bg-blue-400 active:scale-[0.98] text-white text-[13px] font-semibold transition-all"
          @click="store.generate()"
        >
          {{ t('transcript.generate') }}
        </button>

        <p v-if="store.error" class="text-[11px] text-danger/90 break-words">{{ store.error }}</p>
      </div>

      <!-- Transcript -->
      <template v-else>
        <div class="flex items-center gap-2 px-3 py-2 border-b border-fg/5 bg-inset/20">
          <div class="relative flex-1">
            <Search class="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-fg/30" />
            <input
              v-model="query"
              type="text"
              :placeholder="t('transcript.search')"
              class="w-full pl-7 pr-2 py-1.5 rounded-lg bg-fg/[0.07] text-fg/85 text-xs outline-none placeholder:text-fg/25"
            />
          </div>

          <button
            class="p-1.5 rounded text-fg/40 hover:text-fg transition-colors"
            :title="t('transcript.copy')"
            @click="copyAll"
          >
            <Check v-if="copied" class="w-3.5 h-3.5 text-success" />
            <Copy v-else class="w-3.5 h-3.5" />
          </button>

          <button
            class="p-1.5 rounded text-fg/40 hover:text-fg transition-colors"
            :title="t('transcript.again')"
            @click="store.discard()"
          >
            <RefreshCw class="w-3.5 h-3.5" />
          </button>
        </div>

        <div ref="listEl" class="flex-1 overflow-y-auto px-2 py-2 flex flex-col gap-0.5">
          <button
            v-for="item in visibleSegments"
            :key="item.index"
            :ref="(el) => registerRow(item.index, el)"
            :class="[
              'w-full text-left px-2.5 py-1.5 rounded-lg transition-colors flex gap-2.5 items-baseline',
              item.index === store.activeIndex
                ? 'bg-blue-600/20 text-fg'
                : 'text-fg/65 hover:bg-fg/[0.06]',
            ]"
            @click="store.jumpTo(item.segment.start)"
          >
            <span
              class="font-mono text-[10px] shrink-0 pt-0.5"
              :class="item.index === store.activeIndex ? 'text-accent' : 'text-fg/30'"
            >
              {{ formatTime(item.segment.start) }}
            </span>
            <span class="text-[12.5px] leading-relaxed">{{ item.segment.text }}</span>
          </button>

          <div
            v-if="visibleSegments.length === 0"
            class="h-full flex items-center justify-center text-xs text-fg/30"
          >
            {{ t('transcript.noMatches') }}
          </div>
        </div>
      </template>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import {
  Captions,
  X,
  FileText,
  FileAudio,
  Search,
  Copy,
  Check,
  RefreshCw,
  Loader2,
  Download,
} from '@lucide/vue';
import { useTranscriptStore, TRANSCRIPT_LANGUAGES } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useI18n } from '@/composables/useI18n';
import { formatBytes, formatTime } from '@/utils/formatters';

const store = useTranscriptStore();
const { t } = useI18n();
const player = usePlayerStore();

const query = ref('');
const copied = ref(false);
const listEl = ref<HTMLElement | null>(null);
const rows = new Map<number, HTMLElement>();

const hasMedia = computed(() => player.mediaInfo !== null);
// `engine` is null only until the first status call returns; showing the
// download prompt before then would flash it at someone who has the model.
const needsModel = computed(() => store.engine !== null && !store.engine.ready);
const modelSize = computed(() => formatBytes(store.engine?.model_bytes ?? 0));

const downloadLabel = computed(
  () => `${formatBytes(store.downloaded)} / ${formatBytes(store.downloadTotal)}`
);
const downloadPercent = computed(() => {
  if (store.downloadTotal <= 0) return '0%';
  return `${Math.round((store.downloaded / store.downloadTotal) * 100)}%`;
});

const stageLabel = computed(() =>
  store.stage === 'extracting' ? t('transcript.extracting') : t('transcript.transcribing')
);
const percentLabel = computed(() => `${Math.round(Math.max(0, store.progress) * 100)}%`);

const visibleSegments = computed(() => {
  const segments = store.transcript?.segments ?? [];
  const q = query.value.trim().toLowerCase();
  return segments
    .map((segment, index) => ({ segment, index }))
    .filter((item) => q === '' || item.segment.text.toLowerCase().includes(q));
});

function registerRow(index: number, el: unknown) {
  if (el instanceof HTMLElement) {
    rows.set(index, el);
  } else {
    rows.delete(index);
  }
}

// Follow the playhead, but not while the reader is searching for something.
watch(
  () => store.activeIndex,
  async (index) => {
    if (index < 0 || query.value !== '' || !store.isPanelOpen) return;
    await nextTick();
    rows.get(index)?.scrollIntoView({ block: 'center', behavior: 'smooth' });
  }
);

async function copyAll() {
  const segments = store.transcript?.segments ?? [];
  const text = segments.map((s) => `[${formatTime(s.start)}] ${s.text}`).join('\n');

  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    window.setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    console.error('Failed to copy transcript:', e);
  }
}
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
