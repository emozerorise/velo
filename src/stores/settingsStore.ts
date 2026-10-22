import { defineStore } from "pinia";
import { ref } from "vue";
import {
  settingsService,
  type HistoryItem,
  type HistoryData,
} from "@/services/settingsService";
import type {
  AppSettings,
  SummaryFormat,
  TranscriptionDevice,
} from "@/types/settings";
import { setLocale } from "@/composables/useI18n";
import { defaultFont, fontCss, type FontFamily } from "@/utils/fonts";

export type SettingsTab =
  | "general"
  | "video"
  | "subtitles"
  | "transcript"
  | "ai"
  | "updates"
  | "typography";

export const useSettingsStore = defineStore("settings", () => {
  const isSettingsOpen = ref<boolean>(false);
  const isMediaInfoOpen = ref<boolean>(false);
  const activeTab = ref<SettingsTab>("general");
  const settings = ref<AppSettings>({
    version: 1,
    general: {
      font_family: defaultFont,
      theme: "dark",
      language: "en",
      remember_playback_position: true,
      auto_play_next: true,
      show_summary_timestamps: true,
    },
    video: {
      hardware_acceleration: true,
      default_aspect_ratio: "auto",
    },
    audio: {
      default_volume: 80,
      preferred_language: "eng",
      volume_step: 5,
      audio_delay_step: 0.1,
    },
    subtitle: {
      preferred_language: "eng",
      auto_load_external: true,
      font_size: 48,
      subtitle_delay_step: 0.1,
    },
    transcript: {
      device: "auto",
      language: "auto",
      prompt: "",
    },
    summary: {
      format: "meeting",
      provider: "ollama",
      base_url: "http://localhost:11434",
      model: "qwen3:30b-instruct",
      language: "auto",
      instructions: "",
      context_tokens: 32768,
    },
    updates: { auto_check: true, last_checked_at: 0, skipped_version: "" },
  });

  const history = ref<HistoryData>({
    recent_files: [],
    resume_positions: {},
  });

  let settingsWrite: Promise<unknown> = Promise.resolve();
  function serializeWrite<T>(operation: () => Promise<T>): Promise<T> {
    const next = settingsWrite.then(operation);
    settingsWrite = next.catch(() => undefined);
    return next;
  }
  const summaryOptionsSaving = ref(false);
  const fontSaving = ref(false);
  const transcriptDeviceSaving = ref(false);
  async function saveTranscriptDevice(
    device: TranscriptionDevice,
  ): Promise<boolean> {
    if (transcriptDeviceSaving.value) return false;
    transcriptDeviceSaving.value = true;
    return serializeWrite(async () => {
      try {
        await settingsService.save({
          ...settings.value,
          transcript: { ...settings.value.transcript, device },
        });
        settings.value = {
          ...settings.value,
          transcript: { ...settings.value.transcript, device },
        };
        return true;
      } catch {
        return false;
      } finally {
        transcriptDeviceSaving.value = false;
      }
    });
  }
  function applyFont(font: FontFamily) {
    document.documentElement.style.setProperty("--velo-font", fontCss(font));
  }
  async function saveFont(font: FontFamily): Promise<boolean> {
    if (fontSaving.value) return false;
    fontSaving.value = true;
    return serializeWrite(async () => {
      const next = {
        ...settings.value,
        general: { ...settings.value.general, font_family: font },
      };
      try {
        await settingsService.save(next);
        settings.value = {
          ...settings.value,
          general: { ...settings.value.general, font_family: font },
        };
        applyFont(font);
        return true;
      } catch {
        return false;
      } finally {
        fontSaving.value = false;
      }
    });
  }
  async function saveSummaryOptions(options: {
    format?: SummaryFormat;
    showTimestamps?: boolean;
    instructions?: string;
  }): Promise<boolean> {
    if (summaryOptionsSaving.value) return false;
    summaryOptionsSaving.value = true;
    return serializeWrite(async () => {
      const next = {
        ...settings.value,
        general: { ...settings.value.general },
        summary: { ...settings.value.summary },
      };
      if (options.showTimestamps !== undefined)
        next.general.show_summary_timestamps = options.showTimestamps;
      if (options.instructions !== undefined)
        next.summary.instructions = options.instructions;
      if (options.format !== undefined) next.summary.format = options.format;
      try {
        await settingsService.save(next);
        // Commit only these fields after disk succeeds; a failed save must not
        // pretend a new prompt or reading preference will survive a restart.
        if (options.showTimestamps !== undefined)
          settings.value.general.show_summary_timestamps =
            options.showTimestamps;
        if (options.instructions !== undefined)
          settings.value.summary.instructions = options.instructions;
        if (options.format !== undefined)
          settings.value.summary.format = options.format;
        return true;
      } catch (error) {
        console.error("Failed to save summary options:", error);
        return false;
      } finally {
        summaryOptionsSaving.value = false;
      }
    });
  }

  async function loadSettings() {
    try {
      const s = await settingsService.getAll();
      settings.value = s;
      applyFont(s.general.font_family);
      applyTheme(s.general.theme);
      applyLanguage(s.general.language);

      const h = await settingsService.getHistory();
      history.value = h;
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async function saveSettings(newSettings: AppSettings): Promise<boolean> {
    const draft: AppSettings = JSON.parse(JSON.stringify(newSettings));
    // The dedicated processor control owns this field. A modal draft or a
    // job's automatic save must not restore the mode from an older snapshot.
    return serializeWrite(async () => {
      const next = {
        ...draft,
        transcript: {
          ...draft.transcript,
          device: settings.value.transcript.device,
        },
      };
      settings.value = next;
      applyTheme(next.general.theme);
      applyLanguage(next.general.language);
      try {
        await settingsService.save(next);
        return true;
      } catch (e) {
        console.error("Failed to save settings:", e);
        return false;
      }
    });
  }

  function openSettings(tab: SettingsTab = "general") {
    activeTab.value = tab;
    isSettingsOpen.value = true;
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
          (f) => f.path !== item.path,
        );
        history.value.recent_files.unshift(item);
        history.value.resume_positions[item.path] = item.last_position;
      })
      .catch((e: unknown) => {
        console.error("Failed to record history:", e);
      });
  }

  function applyLanguage(language: string) {
    document.documentElement.lang = setLocale(language);
  }

  function applyTheme(theme: string) {
    const root = document.documentElement;
    if (theme === "dark") {
      root.classList.add("dark");
      root.classList.remove("light");
    } else if (theme === "light") {
      root.classList.add("light");
      root.classList.remove("dark");
    } else {
      // System
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      if (prefersDark) {
        root.classList.add("dark");
        root.classList.remove("light");
      } else {
        root.classList.add("light");
        root.classList.remove("dark");
      }
    }
  }

  return {
    isSettingsOpen,
    isMediaInfoOpen,
    activeTab,
    settings,
    history,
    loadSettings,
    saveSettings,
    saveSummaryOptions,
    saveFont,
    saveTranscriptDevice,
    transcriptDeviceSaving,
    fontSaving,
    summaryOptionsSaving,
    openSettings,
    recordPlayback,
    applyTheme,
    applyLanguage,
  };
});
