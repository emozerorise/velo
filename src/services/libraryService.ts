import { invoke } from '@tauri-apps/api/core';
import type { LibraryData, LibraryMutation } from '@/types/library';
export const libraryService = {
  load: () => invoke<LibraryData>('library_load'),
  mutate: (revision: number, mutation: LibraryMutation) => invoke<LibraryData>('library_mutate', {expectedRevision: revision, mutation}),
  exists: (path: string) => invoke<boolean>('library_media_exists', {path}),
};
