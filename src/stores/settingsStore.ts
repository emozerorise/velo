import { defineStore } from 'pinia';
import { ref } from 'vue';
import { settingsService, type HistoryItem, type HistoryData } from '@/services/settingsService';
import type { AppSettings } from '@/types/settings';
import { setLocale } from '@/composables/useI18n';

export const useSettingsStore = defineStore('settings', () => {
  const isSettingsOpen = ref<boolean>(false);
  const isMediaInfoOpen = ref<boolean>(false);
  const settings = ref<AppSettings>({
    version: 1,
    general: {
      theme: 'dark',
      language: 'en',
      remember_playback_position: true,
      auto_play_next: true,
    },
    video: {
      hardware_acceleration: true,
      default_aspect_ratio: 'auto',
    },
    audio: {
      default_volume: 80,
      preferred_language: 'eng',
      volume_step: 5,
      audio_delay_step: 0.1,
    },
    subtitle: {
      preferred_language: 'eng',
      auto_load_external: true,
      font_size: 48,
      subtitle_delay_step: 0.1,
    },
    transcript: {
      language: 'auto',
      prompt: '',
    },
  });

  const history = ref<HistoryData>({
    recent_files: [],
    resume_positions: {},
  });

  async function loadSettings() {
    try {
      const s = await settingsService.getAll();
      settings.value = s;
      applyTheme(s.general.theme);
      applyLanguage(s.general.language);

      const h = await settingsService.getHistory();
      history.value = h;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  function saveSettings(newSettings: AppSettings): void {
    settings.value = newSettings;
    applyTheme(newSettings.general.theme);
    applyLanguage(newSettings.general.language);
    settingsService.save(newSettings).catch((e: unknown) => {
      console.error('Failed to save settings:', e);
    });
  }

  // Fire-and-forget like the playback commands: playback must not wait on a
  // disk write, and a failure here is worth logging, not surfacing.
  function recordPlayback(item: HistoryItem): void {
    settingsService
      .recordHistory(item)
      .then(() => {
        // Mirror what the backend just stored, so the drawer and the resume
        // lookup stay correct without re-fetching.
        history.value.recent_files = history.value.recent_files.filter(
          (f) => f.path !== item.path
        );
        history.value.recent_files.unshift(item);
        history.value.resume_positions[item.path] = item.last_position;
      })
      .catch((e: unknown) => {
        console.error('Failed to record history:', e);
      });
  }

  function applyLanguage(language: string) {
    document.documentElement.lang = setLocale(language);
  }

  function applyTheme(theme: string) {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
      root.classList.remove('light');
    } else if (theme === 'light') {
      root.classList.add('light');
      root.classList.remove('dark');
    } else {
      // System
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      if (prefersDark) {
        root.classList.add('dark');
        root.classList.remove('light');
      } else {
        root.classList.add('light');
        root.classList.remove('dark');
      }
    }
  }

  return {
    isSettingsOpen,
    isMediaInfoOpen,
    settings,
    history,
    loadSettings,
    saveSettings,
    recordPlayback,
    applyTheme,
    applyLanguage,
  };
});
