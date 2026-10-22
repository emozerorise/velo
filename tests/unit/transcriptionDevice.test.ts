import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia, getActivePinia } from 'pinia';
import { nextTick, reactive, watch } from 'vue';
import { mountView } from './helpers/mountView';
import TranscriptionDeviceSettings from '@/components/settings/TranscriptionDeviceSettings.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useToast } from '@/composables/useToast';
import type { MediaInfo } from '@/types/player';
const handlers = new Map<string, (event: { payload: unknown }) => void>();
const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (name: string, args: unknown) => invoke(name, args) }));
vi.mock('@tauri-apps/api/event', () => ({ listen: async (name: string, fn: (event: { payload: unknown }) => void) => {
  handlers.set(name, fn); return () => handlers.delete(name);
} }));
const path = '/meeting.m4a';
const settle = () => new Promise(resolve => setTimeout(resolve, 0));
beforeEach(() => {
  setActivePinia(createPinia()); handlers.clear(); invoke.mockReset();
  invoke.mockResolvedValue({ ready: true, downloading: false });
  vi.stubGlobal('document', { documentElement: { style: { setProperty() {} }, classList: { add() {}, remove() {} } } });
  useToast().toasts.value = [];
  usePlayerStore().mediaInfo = { path } as MediaInfo;
});
afterEach(() => vi.unstubAllGlobals());
it.each(['auto', 'gpu', 'cpu'] as const)('persists %s and uses it for a new transcription', async device => {
  const settings = useSettingsStore();
  const view = mountView(TranscriptionDeviceSettings, {}, getActivePinia()!);
  expect(settings.settings.transcript.device).toBe('auto');
  view.find('transcription-device').trigger('change', { target: { value: device } });
  await settle(); await nextTick();
  expect(settings.settings.transcript.device).toBe(device);
  expect(invoke.mock.calls.some(([name, args]) => name === 'settings_save' && args.settings.transcript.device === device)).toBe(true);
  await useTranscriptStore().generate();
  expect(invoke).toHaveBeenCalledWith('transcript_generate', { path, language: 'auto', prompt: '', device });
  view.unmount();
});
it('keeps the preceding mode and reports a failed settings write', async () => {
  invoke.mockRejectedValueOnce(new Error('disk full'));
  const view = mountView(TranscriptionDeviceSettings, {}, getActivePinia()!);
  view.find('transcription-device').trigger('change', { target: { value: 'cpu' } });
  await settle(); await nextTick();
  expect(useSettingsStore().settings.transcript.device).toBe('auto');
  expect(view.find('transcription-device-error').text.length).toBeGreaterThan(0);
  view.unmount();
});
it('announces CPU fallback, retains the status after completion and clears it for a new file', async () => {
  const store = useTranscriptStore(); await store.initListeners(); await store.generate();
  handlers.get('velo://transcript-device')?.({ payload: { path, device: 'cpu', fallback: 'gpu_failed' } });
  expect(store.execution?.device).toBe('cpu');
  expect(store.execution?.fallback).toBe('gpu_failed');
  expect(useToast().toasts.value.some(t => t.text.includes('CPU'))).toBe(true);
  handlers.get('velo://transcript-ready')?.({ payload: { path, segments: [] } });
  expect(store.execution?.fallback).toBe('gpu_failed');
  usePlayerStore().mediaInfo = { path: '/other.m4a' } as MediaInfo;
  await store.loadForCurrentMedia(); expect(store.execution).toBeNull();
  store.cleanupListeners();
});
it('ignores device events for other files and removes its listener on cleanup', async () => {
  const store = useTranscriptStore(); await store.initListeners();
  handlers.get('velo://transcript-device')?.({ payload: { path: '/other.m4a', device: 'cpu', fallback: 'no_gpu' } });
  expect(store.execution).toBeNull(); expect(useToast().toasts.value).toHaveLength(0);
  store.cleanupListeners(); expect(handlers.has('velo://transcript-device')).toBe(false);
});

it('serializes an overlapping modal save without losing the mode or the other preference', async () => {
  const store = useSettingsStore();
  const staleDraft = JSON.parse(JSON.stringify(store.settings));
  staleDraft.audio.default_volume = 42;
  let finish!: () => void;
  let disk: unknown;
  invoke.mockImplementation(async (name, args) => {
    if (name !== 'settings_save') return;
    if (!finish) await new Promise<void>(resolve => { finish = resolve; });
    disk = JSON.parse(JSON.stringify(args.settings));
  });
  const deviceSave = store.saveTranscriptDevice('cpu');
  await Promise.resolve();
  const modalSave = store.saveSettings(staleDraft);
  expect(invoke.mock.calls.filter(([name]) => name === 'settings_save')).toHaveLength(1);
  finish();
  expect(await deviceSave).toBe(true); expect(await modalSave).toBe(true);
  expect(store.settings.transcript.device).toBe('cpu');
  expect(disk).toMatchObject({ transcript: { device: 'cpu' }, audio: { default_volume: 42 } });
});

it('snapshots a queued reactive modal edit before settings watchers refresh the draft', async () => {
  const store = useSettingsStore();
  const draft = reactive(JSON.parse(JSON.stringify(store.settings)));
  const stop = watch(() => store.settings, value => Object.assign(draft, JSON.parse(JSON.stringify(value))));
  let finish!: () => void; let first = true; let disk: unknown;
  invoke.mockImplementation(async (name, args) => {
    if (name !== 'settings_save') return;
    if (first) { first = false; await new Promise<void>(resolve => { finish = resolve; }); }
    disk = JSON.parse(JSON.stringify(args.settings));
  });
  const mode = store.saveTranscriptDevice('cpu'); await Promise.resolve();
  draft.general.theme = 'light';
  const edited = store.saveSettings(draft);
  finish(); await mode; await edited;
  expect(disk).toMatchObject({ transcript: { device: 'cpu' }, general: { theme: 'light' } });
  stop();
});
