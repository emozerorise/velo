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
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import BaseModal from '@/components/common/BaseModal.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import type { AppSettings } from '@/types/settings';
import { useI18n, SUPPORTED_LOCALES } from '@/composables/useI18n';

const settingsStore = useSettingsStore();
const { t } = useI18n();
const activeTab = ref('general');

// Computed so the labels follow a language change made in this very modal.
const tabs = computed(() => [
  { id: 'general', name: t('settings.tab.general') },
  { id: 'video', name: t('settings.tab.video') },
  { id: 'subtitles', name: t('settings.tab.subtitles') },
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
