import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { libraryService } from '@/services/libraryService';
import { baseName, emptyLibrary, type LibraryMutation, type LibraryData, type LibrarySort } from '@/types/library';

export const useLibraryStore = defineStore('library', () => {
  const snapshot = ref(emptyLibrary());
  const loading = ref(false);
  const error = ref('');
  const loaded = ref(false);
  const unavailable = ref<string[]>([]);
  const renameTarget = ref<{id: string; initial: string} | null>(null);
  const writesEnabled = computed(() => loaded.value && !loading.value);
  let initialization: Promise<void> | undefined;
  let edits: Promise<unknown> = Promise.resolve();
  function message(e: unknown): string { return e && typeof e === 'object' && 'message' in e ? String(e.message) : String(e); }
  async function load() {
    loading.value = true; loaded.value = false;
    try {
      const data = await libraryService.load();
      if (!data || data.version !== 1 || !Array.isArray(data.recordings)) throw new Error('Invalid library data');
      snapshot.value = data; loaded.value = true; error.value = '';
    } catch (e) { error.value = message(e); }
    finally { loading.value = false; }
  }
  function initialize(): Promise<void> { return initialization ??= load(); }
  async function retryLoad(): Promise<void> { await edits; initialization = load(); await initialization; }
  function apply(mutation: LibraryMutation | ((data: LibraryData) => LibraryMutation)): Promise<boolean> {
    const ready = initialize();
    const operation = edits.then(async () => {
      await ready;
      if (!writesEnabled.value) return false;
      try {
        snapshot.value = await libraryService.mutate(snapshot.value.revision, typeof mutation === 'function' ? mutation(snapshot.value) : mutation);
        error.value = ''; return true;
      } catch (e) {
        if (e && typeof e === 'object' && 'kind' in e && e.kind === 'conflict') await load();
        error.value = message(e); return false;
      }
    });
    edits = operation.catch(() => {});
    return operation;
  }
  function displayName(path: string): string { return snapshot.value.recordings.find(r => r.path === path)?.display_name || baseName(path); }
  function beginRename(id: string) {
    const item = snapshot.value.recordings.find(r => r.id === id);
    if (item) renameTarget.value = {id, initial: displayName(item.path)};
  }
  function setSort(sort: LibrarySort) { return apply(data => ({type: 'set_preferences', preferences: {...data.preferences, sort}})); }
  function toggleGroup(id: string) { return apply(data => {
    const ids = data.preferences.collapsed_project_ids;
    return {type: 'set_preferences', preferences: {...data.preferences, collapsed_project_ids: ids.includes(id) ? ids.filter(v => v !== id) : [...ids, id]}};
  }); }
  async function updateDuration(path: string, duration: number): Promise<void> {
    await initialize();
    const recording = snapshot.value.recordings.find(r => r.path === path);
    if (recording && Number.isFinite(duration) && duration >= 0 && recording.duration !== duration) {
      await apply({type: 'set_duration', id: recording.id, duration});
    }
  }
  async function checkAvailable(path: string): Promise<boolean> {
    const exists = await libraryService.exists(path);
    unavailable.value = unavailable.value.filter(p => p !== path);
    if (!exists) unavailable.value.push(path);
    return exists;
  }
  return {snapshot, loading, error, writesEnabled, unavailable, renameTarget, beginRename, setSort, toggleGroup, initialize, retryLoad, apply, displayName, checkAvailable, updateDuration};
});
