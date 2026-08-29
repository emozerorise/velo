<template>
  <Modal
    :is-open="settingsStore.isMediaInfoOpen ? false : settingsStore.isSettingsOpen"
    title="Preferences"
    @close="settingsStore.isSettingsOpen = false"
  >
    <div class="flex flex-col gap-6 text-sm">
      <!-- Tabs Navigation -->
      <div class="flex items-center gap-2 border-b border-white/10 pb-2 text-xs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="[
            'px-3 py-1.5 rounded-lg transition-colors font-medium',
            activeTab === tab.id
              ? 'bg-blue-600/30 text-blue-400 border border-blue-500/30'
              : 'text-white/60 hover:text-white hover:bg-white/5',
          ]"
          @click="activeTab = tab.id"
        >
          {{ tab.name }}
        </button>
      </div>

      <!-- Tab Content: General -->
      <div v-if="activeTab === 'general'" class="flex flex-col gap-4">
        <!-- Theme -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-white/90">Appearance Theme</span>
            <span class="block text-xs text-white/50">Select interface color mode</span>
          </div>
          <select
            v-model="localSettings.general.theme"
            class="bg-black/40 border border-white/15 rounded-lg px-3 py-1.5 text-xs text-white outline-none"
            @change="save"
          >
            <option value="dark">Dark Theme</option>
            <option value="light">Light Theme</option>
            <option value="system">System Default</option>
          </select>
        </div>

        <!-- Remember Playback Position -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-white/90">Resume Playback</span>
            <span class="block text-xs text-white/50">Remember last watched position in recent media</span>
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
            <span class="font-medium text-white/90">Auto-Play Next</span>
            <span class="block text-xs text-white/50">Automatically play the next item in the playlist</span>
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
            <span class="font-medium text-white/90">Hardware Acceleration</span>
            <span class="block text-xs text-white/50">Enable GPU decoding (VideoToolbox / D3D11VA)</span>
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
            <span class="font-medium text-white/90">Default Aspect Ratio</span>
            <span class="block text-xs text-white/50">Default video viewport ratio</span>
          </div>
          <select
            v-model="localSettings.video.default_aspect_ratio"
            class="bg-black/40 border border-white/15 rounded-lg px-3 py-1.5 text-xs text-white outline-none"
            @change="save"
          >
            <option value="auto">Auto / Source</option>
            <option value="16:9">16:9 Widescreen</option>
            <option value="4:3">4:3 Standard</option>
            <option value="2.35:1">2.35:1 Cinematic</option>
          </select>
        </div>
      </div>

      <!-- Tab Content: Subtitles -->
      <div v-if="activeTab === 'subtitles'" class="flex flex-col gap-4">
        <!-- Auto Load External -->
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium text-white/90">Auto-load Subtitles</span>
            <span class="block text-xs text-white/50">Automatically load matching .srt/.ass subtitle files</span>
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
            <span class="font-medium text-white/90">Subtitle Font Size</span>
            <span class="block text-xs text-white/50">Base size for simple text subtitles</span>
          </div>
          <input
            v-model.number="localSettings.subtitle.font_size"
            type="number"
            min="20"
            max="100"
            class="w-20 bg-black/40 border border-white/15 rounded-lg px-3 py-1 text-xs text-white outline-none font-mono"
            @change="save"
          />
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import Modal from '@/components/common/Modal.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import type { AppSettings } from '@/types/settings';

const settingsStore = useSettingsStore();
const activeTab = ref('general');

const tabs = [
  { id: 'general', name: 'General' },
  { id: 'video', name: 'Video' },
  { id: 'subtitles', name: 'Subtitles' },
];

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
