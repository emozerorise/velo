<template>
  <BaseModal
    :is-open="settingsStore.isMediaInfoOpen ? false : settingsStore.isSettingsOpen"
    :title="t('settings.title')"
    @close="settingsStore.isSettingsOpen = false"
  >
    <div class="flex flex-col gap-6 text-sm">
      <!-- Tabs Navigation -->
      <div class="flex items-center gap-2 border-b border-fg/10 pb-2 text-xs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="[
            'px-3 py-1.5 rounded-lg transition-colors font-medium',
            activeTab === tab.id
              ? 'bg-blue-600/30 text-accent border border-blue-500/30'
              : 'text-fg/60 hover:text-fg hover:bg-fg/5',
          ]"
          @click="activeTab = tab.id"
        >
          {{ tab.name }}
        </button>
      </div>

      <!-- Tab Content: General -->
      <div v-if="activeTab === 'general'" class="flex flex-col gap-4">
        <!-- Interface Language -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.language') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.languageDesc') }}</span>
          </div>
          <select
            v-model="localSettings.general.language"
            class="bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none"
            @change="save"
          >
            <option v-for="option in SUPPORTED_LOCALES" :key="option.code" :value="option.code">
              {{ option.label }}
            </option>
          </select>
        </div>

        <!-- Theme -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.theme') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.themeDesc') }}</span>
          </div>
          <select
            v-model="localSettings.general.theme"
            class="bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none"
            @change="save"
          >
            <option value="dark">{{ t('settings.theme.dark') }}</option>
            <option value="light">{{ t('settings.theme.light') }}</option>
            <option value="system">{{ t('settings.theme.system') }}</option>
          </select>
        </div>

        <!-- Remember Playback Position -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.resume') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.resumeDesc') }}</span>
          </div>
          <input
            v-model="localSettings.general.remember_playback_position"
            type="checkbox"
            class="w-4 h-4 rounded accent-blue-500"
            @change="save"
          />
        </div>

        <!-- Auto-Play Next -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.autoPlayNext') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.autoPlayNextDesc') }}</span>
          </div>
          <input
            v-model="localSettings.general.auto_play_next"
            type="checkbox"
            class="w-4 h-4 rounded accent-blue-500"
            @change="save"
          />
        </div>
      </div>

      <!-- Tab Content: Video -->
      <div v-if="activeTab === 'video'" class="flex flex-col gap-4">
        <!-- Hardware Acceleration -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.hwaccel') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.hwaccelDesc') }}</span>
          </div>
          <input
            v-model="localSettings.video.hardware_acceleration"
            type="checkbox"
            class="w-4 h-4 rounded accent-blue-500"
            @change="save"
          />
        </div>

        <!-- Aspect Ratio -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.aspectRatio') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.aspectRatioDesc') }}</span>
          </div>
          <select
            v-model="localSettings.video.default_aspect_ratio"
            class="bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none"
            @change="save"
          >
            <option value="auto">{{ t('settings.aspect.auto') }}</option>
            <option value="16:9">{{ t('settings.aspect.16:9') }}</option>
            <option value="4:3">{{ t('settings.aspect.4:3') }}</option>
            <option value="2.35:1">{{ t('settings.aspect.2.35:1') }}</option>
          </select>
        </div>
      </div>

      <!-- Tab Content: Subtitles -->
      <div v-if="activeTab === 'subtitles'" class="flex flex-col gap-4">
        <!-- Auto Load External -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.autoLoadSubs') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.autoLoadSubsDesc') }}</span>
          </div>
          <input
            v-model="localSettings.subtitle.auto_load_external"
            type="checkbox"
            class="w-4 h-4 rounded accent-blue-500"
            @change="save"
          />
        </div>

        <!-- Font Size -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.subFontSize') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.subFontSizeDesc') }}</span>
          </div>
          <input
            v-model.number="localSettings.subtitle.font_size"
            type="number"
            min="20"
            max="100"
            class="w-20 bg-inset/40 border border-fg/15 rounded-lg px-3 py-1 text-xs text-fg outline-none font-mono"
            @change="save"
          />
        </div>
      </div>

      <!-- Tab Content: Transcription -->
      <div v-if="activeTab === 'transcript'" class="flex flex-col gap-4">
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <span class="font-medium text-fg/90">{{ t('settings.model') }}</span>
            <span class="block text-xs text-fg/50">{{ modelStatus }}</span>
            <span v-if="isExternalModel" class="block text-xs text-fg/40 mt-1">
              {{ t('settings.modelExternalDesc') }}
            </span>
          </div>

          <!-- Two-step rather than a dialog: half a gigabyte is worth a
               confirmation, but not a modal on top of a modal. -->
          <div v-if="canRemoveModel" class="shrink-0">
            <button
              v-if="!confirmingRemove"
              class="px-3 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] text-fg/80 text-xs transition-colors"
              @click="confirmingRemove = true"
            >
              {{ t('settings.modelRemove') }}
            </button>

            <div v-else class="flex items-center gap-1.5">
              <button
                class="px-3 py-1.5 rounded-lg bg-red-500/15 hover:bg-red-500/25 text-danger text-xs font-medium transition-colors"
                @click="removeModel"
              >
                {{ t('settings.modelRemoveYes') }}
              </button>
              <button
                class="px-3 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] text-fg/80 text-xs transition-colors"
                @click="confirmingRemove = false"
              >
                {{ t('settings.modelRemoveNo') }}
              </button>
            </div>
          </div>
        </div>

        <p v-if="canRemoveModel" class="text-xs text-fg/40 leading-relaxed">
          {{ t('settings.modelRemoveDesc') }}
        </p>

        <p v-if="transcriptStore.modelError" class="text-xs text-danger/90 break-words">
          {{ transcriptStore.modelError }}
        </p>
      </div>

      <!-- Tab Content: AI -->
      <div v-if="activeTab === 'ai'" class="flex flex-col gap-4">
        <!-- Provider -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.aiProvider') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.aiProviderDesc') }}</span>
          </div>
          <select
            v-model="localSettings.summary.provider"
            class="bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none"
            @change="onProviderChange"
          >
            <option value="ollama">{{ t('settings.aiProvider.ollama') }}</option>
            <option value="openai">{{ t('settings.aiProvider.openai') }}</option>
          </select>
        </div>

        <!-- Server Address -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <span class="font-medium text-fg/90">{{ t('settings.aiBaseUrl') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.aiBaseUrlDesc') }}</span>
          </div>
          <input
            v-model="localSettings.summary.base_url"
            type="text"
            spellcheck="false"
            class="w-52 shrink-0 bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none font-mono"
            @change="save"
          />
        </div>

        <!-- Model, offered from what the server actually has -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <span class="font-medium text-fg/90">{{ t('settings.aiModel') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.aiModelDesc') }}</span>
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
            <input
              v-model="localSettings.summary.model"
              type="text"
              list="velo-summary-models"
              spellcheck="false"
              class="w-36 bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none font-mono"
              @change="save"
            />
            <datalist id="velo-summary-models">
              <option v-for="name in summaryStore.models" :key="name" :value="name" />
            </datalist>
            <button
              class="px-3 py-1.5 rounded-lg bg-fg/[0.07] hover:bg-fg/[0.12] text-fg/80 text-xs transition-colors"
              :disabled="testing"
              @click="testConnection"
            >
              {{ testing ? t('settings.aiTesting') : t('settings.aiTest') }}
            </button>
          </div>
        </div>

        <p
          v-if="testResult"
          class="text-xs break-words"
          :class="testOk ? 'text-success' : 'text-danger/90'"
        >
          {{ testResult }}
        </p>

        <!-- Summary Language -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-fg/90">{{ t('settings.aiLanguage') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.aiLanguageDesc') }}</span>
          </div>
          <select
            v-model="localSettings.summary.language"
            class="bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none"
            @change="save"
          >
            <option v-for="option in SUMMARY_LANGUAGES" :key="option.code" :value="option.code">
              {{ t(option.key) }}
            </option>
          </select>
        </div>

        <!-- Context Window -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <span class="font-medium text-fg/90">{{ t('settings.aiContext') }}</span>
            <span class="block text-xs text-fg/50">{{ t('settings.aiContextDesc') }}</span>
          </div>
          <input
            v-model.number="localSettings.summary.context_tokens"
            type="number"
            min="2048"
            max="131072"
            step="2048"
            class="w-24 shrink-0 bg-inset/40 border border-fg/15 rounded-lg px-3 py-1 text-xs text-fg outline-none font-mono"
            @change="save"
          />
        </div>

        <!-- Extra Instructions -->
        <div>
          <span class="font-medium text-fg/90">{{ t('settings.aiInstructions') }}</span>
          <span class="block text-xs text-fg/50 mb-1.5">
            {{ t('settings.aiInstructionsDesc') }}
          </span>
          <textarea
            v-model="localSettings.summary.instructions"
            rows="2"
            :placeholder="t('settings.aiInstructionsPlaceholder')"
            class="w-full bg-inset/40 border border-fg/15 rounded-lg px-3 py-2 text-xs text-fg outline-none resize-none placeholder:text-fg/25"
            @change="save"
          />
        </div>

        <p
          class="text-xs leading-relaxed"
          :class="isRemoteProvider ? 'text-amber-500/90' : 'text-fg/40'"
        >
          {{
            isRemoteProvider
              ? t('settings.aiPrivacyRemote', { host: providerHost })
              : t('settings.aiPrivacyLocal')
          }}
        </p>

        <p
          v-if="localSettings.summary.provider === 'openai'"
          class="text-xs text-fg/40 leading-relaxed"
        >
          {{ t('settings.aiKeyMissing') }}
        </p>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import BaseModal from '@/components/common/BaseModal.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import type { AppSettings } from '@/types/settings';
import { useI18n, SUPPORTED_LOCALES } from '@/composables/useI18n';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSummaryStore, SUMMARY_LANGUAGES } from '@/stores/summaryStore';
import { useToast } from '@/composables/useToast';
import { formatBytes } from '@/utils/formatters';

const settingsStore = useSettingsStore();
const transcriptStore = useTranscriptStore();
const summaryStore = useSummaryStore();
const { t } = useI18n();
const { showToast } = useToast();
const activeTab = ref('general');
const confirmingRemove = ref(false);

const isExternalModel = computed(
  () => transcriptStore.engine?.ready === true && !transcriptStore.engine.model_managed
);

// Only the app's own downloaded copy may be deleted; an override points at a
// file the user owns.
const canRemoveModel = computed(() => transcriptStore.engine?.model_managed === true);

const modelStatus = computed(() => {
  const engine = transcriptStore.engine;
  if (!engine || !engine.ready) return t('settings.modelAbsent');
  if (!engine.model_managed) return t('settings.modelExternal');
  return t('settings.modelDownloaded', { size: formatBytes(engine.model_bytes) });
});

const testing = ref(false);
const testOk = ref(false);
const testResult = ref<string | null>(null);

const providerHost = computed(() => {
  try {
    return new URL(localSettings.summary.base_url).host;
  } catch {
    return localSettings.summary.base_url;
  }
});

const isRemoteProvider = computed(() => {
  const host = providerHost.value.split(':')[0];
  return !['localhost', '127.0.0.1', '0.0.0.0', '[', '::1'].some((local) => host.startsWith(local));
});

// The two dialects disagree about whether the version segment belongs in the
// address, so switching preset fills in the one that provider expects.
function onProviderChange() {
  localSettings.summary.base_url =
    localSettings.summary.provider === 'openai'
      ? 'https://api.openai.com/v1'
      : 'http://localhost:11434';
  summaryStore.models = [];
  testResult.value = null;
  save();
}

/// The backend probes with the settings on disk, so this saves first.
async function testConnection() {
  testing.value = true;
  testResult.value = null;
  save();

  try {
    testOk.value = await summaryStore.probe();
    testResult.value = testOk.value
      ? t('settings.aiFound', { count: summaryStore.models.length })
      : (summaryStore.error ?? '');
  } finally {
    testing.value = false;
    void summaryStore.refreshStatus();
  }
}

async function removeModel() {
  confirmingRemove.value = false;
  const freed = await transcriptStore.removeModel();
  if (freed > 0) {
    showToast(t('settings.modelRemoved', { size: formatBytes(freed) }));
  }
}

// Computed so the labels follow a language change made in this very modal.
const tabs = computed(() => [
  { id: 'general', name: t('settings.tab.general') },
  { id: 'video', name: t('settings.tab.video') },
  { id: 'subtitles', name: t('settings.tab.subtitles') },
  { id: 'transcript', name: t('settings.tab.transcript') },
  { id: 'ai', name: t('settings.tab.ai') },
]);

const localSettings = reactive<AppSettings>(JSON.parse(JSON.stringify(settingsStore.settings)));

watch(
  () => settingsStore.settings,
  (newVal) => {
    Object.assign(localSettings, JSON.parse(JSON.stringify(newVal)));
  }
);

function save() {
  settingsStore.saveSettings(localSettings);
}
</script>
