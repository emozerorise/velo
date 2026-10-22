import { beforeEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { ingestMedia, playLibraryRecording } from '@/services/mediaIngestion';
import { useLibraryStore } from '@/stores/libraryStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { emptyLibrary } from '@/types/library';
const ipc=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>ipc);
beforeEach(()=>{setActivePinia(createPinia());ipc.invoke.mockReset().mockImplementation(async(command:string,args:Record<string,unknown>)=>{
    if(command==='library_load') return emptyLibrary();
    if(command==='library_media_exists') return true;
    if(command==='library_mutate') return {...emptyLibrary(),revision:Number(args.expectedRevision)+1};
});});
it('selects a library path once without re-registering or changing queue order',async()=>{
    const queue=usePlaylistStore();queue.addFiles(['/first.mp4','/second.mp4']);ipc.invoke.mockClear();
    await playLibraryRecording('/second.mp4');
    expect(queue.currentItem?.path).toBe('/second.mp4');expect(queue.items.map(r=>r.path)).toEqual(['/first.mp4','/second.mp4']);
    expect(ipc.invoke.mock.calls.filter(c=>c[0]==='player_load_file')).toHaveLength(1);
    expect(ipc.invoke.mock.calls.some(c=>c[0]==='library_mutate')).toBe(false);
});
it('new library selection does not autoplay twice',async()=>{
    await playLibraryRecording('/new.mp4');
    expect(ipc.invoke.mock.calls.filter(c=>c[0]==='player_load_file')).toHaveLength(1);
});
it('failed persistence still permits direct playback without claiming save',async()=>{
    ipc.invoke.mockImplementation(async(command:string)=>{if(command==='library_load')throw new Error('corrupt');});
    await ingestMedia(['/a.mp4']);expect(usePlaylistStore().currentItem?.path).toBe('/a.mp4');
    expect(useLibraryStore().writesEnabled).toBe(false);
});
it('missing recording is retained and does not start playback',async()=>{
    ipc.invoke.mockResolvedValueOnce(false);expect(await playLibraryRecording('/missing.mp4')).toBe(false);
    expect(useLibraryStore().unavailable).toEqual(['/missing.mp4']);expect(usePlaylistStore().items).toEqual([]);
});
