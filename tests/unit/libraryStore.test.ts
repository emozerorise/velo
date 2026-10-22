import { beforeEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useLibraryStore } from '@/stores/libraryStore';
const ipc=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>ipc);
const snapshot=()=>({version:1,revision:2,projects:[],recordings:[{id:'a',path:'/a.mp4',display_name:'ประชุม',project_id:null,pinned:false,added_at:1,duration:null}],preferences:{collapsed_project_ids:[],sort:'added'}});
beforeEach(()=>{setActivePinia(createPinia());ipc.invoke.mockReset();});
it('waits for initial disk data before applying edits',async()=>{
    let resolve!:(v:unknown)=>void;
    ipc.invoke.mockImplementationOnce(()=>new Promise(r=>{resolve=r;})).mockResolvedValueOnce({...snapshot(),revision:3});
    const store=useLibraryStore();const pending=store.apply({type:'set_pin',id:'a',pinned:true});
    expect(ipc.invoke).toHaveBeenCalledTimes(1);resolve(snapshot());await pending;
    expect(ipc.invoke).toHaveBeenLastCalledWith('library_mutate',{expectedRevision:2,mutation:{type:'set_pin',id:'a',pinned:true}});
    expect(store.displayName('/a.mp4')).toBe('ประชุม');
});
it('failed edit preserves the confirmed snapshot',async()=>{
    ipc.invoke.mockResolvedValueOnce(snapshot()).mockRejectedValueOnce({kind:'storage',message:'disk full'});
    const store=useLibraryStore();await store.initialize();
    expect(await store.apply({type:'rename_recording',id:'a',name:'new'})).toBe(false);
    expect(store.displayName('/a.mp4')).toBe('ประชุม');expect(store.snapshot.revision).toBe(2);
});
it('reloads conflicts without replaying the edit',async()=>{
    ipc.invoke.mockResolvedValueOnce(snapshot()).mockRejectedValueOnce({kind:'conflict',message:'stale'}).mockResolvedValueOnce({...snapshot(),revision:5});
    const store=useLibraryStore();await store.initialize();
    expect(await store.apply({type:'remove_recording',id:'a'})).toBe(false);
    expect(store.snapshot.revision).toBe(5);expect(store.snapshot.recordings).toHaveLength(1);
    expect(ipc.invoke).toHaveBeenCalledTimes(3);
});
it('failed load disables writes and resolves initialization',async()=>{
    ipc.invoke.mockRejectedValue({kind:'storage',message:'bad file'});
    const store=useLibraryStore();await store.initialize();expect(store.writesEnabled).toBe(false);
    expect(await store.apply({type:'remove_recording',id:'a'})).toBe(false);
    expect(ipc.invoke).toHaveBeenCalledTimes(1);
});
it('duration updates never re-add a removed recording',async()=>{
  ipc.invoke.mockResolvedValueOnce(snapshot()).mockResolvedValueOnce({...snapshot(),revision:3});
  const store=useLibraryStore();await store.initialize();
  await store.updateDuration('/missing.mp4',10);expect(ipc.invoke).toHaveBeenCalledTimes(1);
  await store.updateDuration('/a.mp4',42);
  expect(ipc.invoke).toHaveBeenLastCalledWith('library_mutate',{expectedRevision:2,mutation:{type:'set_duration',id:'a',duration:42}});
});
it('queued preference edits compose against the last confirmed snapshot',async()=>{
  const data={...snapshot(),projects:[{id:'p',name:'Team',created_at:1}]};
  ipc.invoke.mockResolvedValueOnce(data).mockImplementation(async(_cmd:string,args:{mutation:{preferences:unknown}})=>({...data,revision:3,preferences:args.mutation.preferences}));
  const store=useLibraryStore();await store.initialize();
  await Promise.all([store.toggleGroup('p'),store.setSort('name')]);
  expect(store.snapshot.preferences).toEqual({collapsed_project_ids:['p'],sort:'name'});
});
