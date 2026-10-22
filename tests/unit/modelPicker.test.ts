import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia, getActivePinia } from 'pinia';
import { nextTick } from 'vue';
import ModelPicker from '@/components/settings/ModelPicker.vue';
import { mountView } from './helpers/mountView';
import { setLocale } from '@/composables/useI18n';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '@/stores/settingsStore';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
afterEach(() => vi.unstubAllGlobals());

beforeEach(() => { setActivePinia(createPinia()); setLocale('en'); });
function mount(model = 'saved-model', models: string[] = []) {
  const change = vi.fn();
  const view = mountView(ModelPicker, { model, models, onChange: change }, getActivePinia()!);
  return { view, change };
}
it('offers every discovered model and emits the selected name, not the option token', async () => {
  const { view, change } = mount('flash', ['flash', 'pro']);
  expect(view.find('model-select').tagName).toBe('SELECT');
  expect(view.nodes().filter(n => n.tagName === 'OPTION').map(n => n.text)).toEqual(['flash', 'pro', 'Enter model name…']);
  view.find('model-select').trigger('change', { target: { value: 'model:pro' } });
  await nextTick();
  expect(change).toHaveBeenCalledWith('pro');
  view.unmount();
});
it('keeps an undiscovered saved model visible without changing it when discovery completes', async () => {
  const { view, change } = mount();
  expect(view.find('model-select').props.value).toBe('model:saved-model');
  expect(view.nodes().some(n => n.text.includes('saved-model'))).toBe(true);
  view.updateProps({ models: ['another-model'] });
  await nextTick();
  expect(view.find('model-select').props.value).toBe('model:saved-model');
  expect(change).not.toHaveBeenCalled();
  view.unmount();
});
it('opens and focuses manual entry without saving a sentinel, trims valid names and rejects blank names', async () => {
  const { view, change } = mount('flash', ['flash']);
  view.find('model-select').trigger('change', { target: { value: 'custom' } });
  await nextTick();
  await nextTick();
  expect(change).not.toHaveBeenCalled();
  expect(view.find('model-custom').focusCount).toBeGreaterThan(0);
  view.find('model-custom').trigger('change', { target: { value: '   ' } });
  await nextTick();
  expect(change).not.toHaveBeenCalled();
  expect(view.find('model-custom').props['aria-invalid']).toBe(true);
  view.find('model-custom').trigger('change', { target: { value: ' my-model:v2 ' } });
  await nextTick();
  expect(change).toHaveBeenCalledWith('my-model:v2');
  view.updateProps({ model: 'my-model:v2' });
  await nextTick();
  expect(view.find('model-select').props.value).toBe('custom');
  view.find('model-select').trigger('change', { target: { value: 'model:flash' } });
  await nextTick();
  expect(view.nodes().some(n => n.props['data-testid'] === 'model-custom')).toBe(false);
  view.unmount();
});
it('offers a placeholder for empty configuration without automatically selecting a discovered model', () => {
  const { view, change } = mount('', ['flash']);
  expect(view.find('model-select').props.value).toBe('');
  expect(change).not.toHaveBeenCalled();
  view.unmount();
});
it('disables selection and manual input while probing', async () => {
  const { view } = mount();
  view.find('model-select').trigger('change', { target: { value: 'custom' } });
  view.updateProps({ disabled: true });
  await nextTick();
  expect(view.find('model-select').props.disabled).toBe(true);
  expect(view.find('model-custom').props.disabled).toBe(true);
  view.unmount();
});
it.each(['en', 'th'])('labels the selector and manual input accessibly in %s', async locale => {
  setLocale(locale);
  const { view } = mount();
  expect(view.find('model-select').props['aria-label']).toBe(locale === 'th' ? 'โมเดล' : 'Model');
  view.find('model-select').trigger('change', { target: { value: 'custom' } });
  await nextTick();
  expect(view.find('model-custom').props['aria-label']).toBe(locale === 'th' ? 'ชื่อโมเดลที่ระบุเอง' : 'Custom model name');
  view.unmount();
});

it('restores a manually selected model after saving and reloading settings', async () => {
  vi.stubGlobal('document', { documentElement: { style: { setProperty() {} }, classList: { add() {}, remove() {} } } });
  const store = useSettingsStore();
  const saved = JSON.parse(JSON.stringify(store.settings));
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === 'settings_save') Object.assign(saved, JSON.parse(JSON.stringify((args as { settings: unknown }).settings)));
    if (command === 'settings_get_all') return saved;
    if (command === 'history_get') return { recent_files: [], resume_positions: {} };
  });
  const { view, change } = mount('flash');
  view.find('model-select').trigger('change', { target: { value: 'custom' } });
  await nextTick();
  view.find('model-custom').trigger('change', { target: { value: 'my-private-model' } });
  await store.saveSettings({ ...store.settings, summary: { ...store.settings.summary, model: change.mock.calls[0][0] } });
  view.unmount();
  setActivePinia(createPinia());
  await useSettingsStore().loadSettings();
  const reopened = mount(useSettingsStore().settings.summary.model);
  expect(reopened.view.find('model-select').props.value).toBe('model:my-private-model');
  reopened.view.unmount();
});
