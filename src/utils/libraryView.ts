import { baseName, type LibraryData, type LibraryRecording } from '@/types/library';
import type { HistoryItem } from '@/services/settingsService';
export function recordingName(recording: LibraryRecording): string { return recording.display_name || baseName(recording.path); }
export function visibleLibrary(data: LibraryData, query: string, history: HistoryItem[]) {
  const search = query.trim().toLocaleLowerCase();
  const played = new Map(history.map(r => [r.path, r.last_played_timestamp]));
  const collator = new Intl.Collator(undefined, {numeric: true, sensitivity: 'base'});
  const matches = (r: LibraryRecording) => !search || `${recordingName(r)} ${baseName(r.path)}`.toLocaleLowerCase().includes(search);
  const sorted = [...data.recordings].sort((a, b) => {
    const order = data.preferences.sort === 'name' ? collator.compare(recordingName(a), recordingName(b)) : data.preferences.sort === 'played' ? (played.get(b.path) ?? 0) - (played.get(a.path) ?? 0) : b.added_at - a.added_at;
    return order || a.id.localeCompare(b.id);
  });
  return {
    pinned: sorted.filter(r => r.pinned && matches(r)),
    groups: [{id: 'uncategorized', name: ''}, ...data.projects].map(p => {
      const all = sorted.filter(r => (r.project_id ?? 'uncategorized') === p.id);
      const items = all.filter(matches);
      return {id: p.id, name: p.name, collapsed: !search && data.preferences.collapsed_project_ids.includes(p.id), matching: items.length, total: all.length, items};
    }),
  };
}
