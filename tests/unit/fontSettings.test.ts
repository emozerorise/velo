import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia, getActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { mountView } from './helpers/mountView';
import FontSettings from '@/components/settings/FontSettings.vue';
import { fontChoices, fontCss } from '@/utils/fonts';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '@/stores/settingsStore';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const setProperty = vi.fn();
beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  vi.stubGlobal('document', { documentElement: { style: { setProperty } } });
});
afterEach(() => vi.unstubAllGlobals());
it('previews all six fonts without saving; discards drafts on remount', async () => {
  const view = mountView(FontSettings, {}, getActivePinia()!);
  for (const font of fontChoices) {
    view.find(`font-${font.id}`).trigger('change');
    await nextTick();
    expect(view.find('font-preview').props.style).toEqual({ fontFamily: fontCss(font.id) });
    expect(view.find('font-preview-name').text).toBe(font.name);
  }
  expect(invoke).not.toHaveBeenCalled();
  expect(setProperty).not.toHaveBeenCalled();
  view.unmount();
  const reopened = mountView(FontSettings, {}, getActivePinia()!);
  expect(reopened.find('font-preview-name').text).toBe('Noto Sans Thai Looped');
  reopened.unmount();
});
it('applies a choice and previews reset without silently saving it', async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  const view = mountView(FontSettings, {}, getActivePinia()!);
  view.find('font-sarabun').trigger('change');
  await nextTick();
  view.find('font-apply').trigger('click');
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(useSettingsStore().settings.general.font_family).toBe('sarabun');
  view.find('font-reset').trigger('click');
  await nextTick();
  expect(view.find('font-preview-name').text).toBe('Noto Sans Thai Looped');
  expect(useSettingsStore().settings.general.font_family).toBe('sarabun');
  expect(invoke).toHaveBeenCalledOnce();
  view.find('font-apply').trigger('click');
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(useSettingsStore().settings.general.font_family).toBe('noto_sans_thai_looped');
  expect(setProperty).toHaveBeenLastCalledWith('--velo-font', '"Noto Sans Thai Looped"');
  view.unmount();
});
it('starts with the looped font and applies changes only after persistence', async () => {
  const store = useSettingsStore();
  expect(store.settings.general.font_family).toBe('noto_sans_thai_looped');
  let finish!: () => void;
  vi.mocked(invoke).mockImplementationOnce(() => new Promise<void>(resolve => { finish = resolve; }));
  const saving = store.saveFont('prompt');
  expect(store.settings.general.font_family).toBe('noto_sans_thai_looped');
  expect(setProperty).not.toHaveBeenCalled();
  finish();
  expect(await saving).toBe(true);
  expect(store.settings.general.font_family).toBe('prompt');
  expect(setProperty).toHaveBeenCalledWith('--velo-font', '"Prompt"');
});
it('preserves the font and other settings after a failed save', async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error('disk full'));
  const store = useSettingsStore();
  const before = JSON.stringify(store.settings);
  expect(await store.saveFont('kanit')).toBe(false);
  expect(JSON.stringify(store.settings)).toBe(before);
  expect(setProperty).not.toHaveBeenCalled();
});
it.each([...fontChoices, { id: 'noto_sans_thai' as const, name: 'Noto Sans Thai' }])('restores $name when settings are loaded again', async (font) => {
  const store = useSettingsStore();
  const saved = JSON.parse(JSON.stringify(store.settings));
  saved.general.font_family = font.id;
  vi.stubGlobal('document', { documentElement: { style: { setProperty }, classList: { add() {}, remove() {} } } });
  vi.mocked(invoke).mockResolvedValueOnce(saved).mockResolvedValueOnce({ recent_files: [], resume_positions: {} });
  await store.loadSettings();
  expect(store.settings.general.font_family).toBe(font.id);
  expect(setProperty).toHaveBeenCalledWith('--velo-font', fontCss(font.id));
});
