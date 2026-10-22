export type LibrarySort = 'added' | 'played' | 'name';
export interface LibraryProject { id: string; name: string; created_at: number }
export interface LibraryRecording { id: string; path: string; display_name: string | null; project_id: string | null; pinned: boolean; added_at: number; duration: number | null }
export interface LibraryPreferences { collapsed_project_ids: string[]; sort: LibrarySort }
export interface LibraryData { version: number; revision: number; projects: LibraryProject[]; recordings: LibraryRecording[]; preferences: LibraryPreferences }
export type LibraryMutation =
  | {type: 'add_recordings'; entries: {id: string; path: string}[]; project_id: string | null}
  | {type: 'create_project' | 'rename_project'; id: string; name: string}
  | {type: 'delete_project' | 'reset_name' | 'remove_recording'; id: string}
  | {type: 'rename_recording'; id: string; name: string}
  | {type: 'move_recording'; id: string; project_id: string | null}
  | {type: 'set_pin'; id: string; pinned: boolean}
  | {type: 'set_preferences'; preferences: LibraryPreferences}
  | {type: 'set_duration'; id: string; duration: number};
export function emptyLibrary(): LibraryData { return {version: 1, revision: 0, projects: [], recordings: [], preferences: {collapsed_project_ids: [], sort: 'added'}}; }
export function baseName(path: string): string { return path.split(/[\\/]/).pop() || path; }
