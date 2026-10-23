import { setLocale } from '@/composables/useI18n';
import { beforeEach, expect, it, vi } from 'vitest';
import { createPinia } from 'pinia';
import { nextTick } from 'vue';
import { mountView, TestNode } from './helpers/mountView';
import LibraryNameEditor from '@/components/library/LibraryNameEditor.vue';
import LibraryGroup from '@/components/library/LibraryGroup.vue';
import LibraryBrowser from '@/components/library/LibraryBrowser.vue';
import ReadingSidebar from '@/components/playlist/ReadingSidebar.vue';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { useLibraryStore } from '@/stores/libraryStore';
import { emptyLibrary } from '@/types/library';
beforeEach(() => setLocale('en'));
const ipc=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>ipc);
it.each(['create','project'])('pointer-opened %s editor restores focus even when Safari does not focus clicks',async(kind)=>{
  vi.stubGlobal('document',{activeElement:null,addEventListener(){},removeEventListener(){}});
  try {
    const pinia=createPinia();const view=kind==='create'
      ? mountView(LibraryBrowser,{},pinia)
      : mountView(LibraryGroup,{group:{id:'p',name:'Team',collapsed:false,matching:0,total:0,items:[]}},pinia);
    if(kind==='project') {view.nodes().find(n=>n.props['aria-label']==='Actions: Team')!.trigger('click');await nextTick();}
    const opener=view.nodes().find(n=>kind==='create' ? n.props['aria-label']==='New project' : n.tagName==='BUTTON' && n.text==='Rename project')!;
    opener.trigger('click',{currentTarget:opener});await nextTick();
    view.find('name-cancel').trigger('click');await nextTick();
    expect(document.activeElement === (opener as unknown as Element)).toBe(true);view.unmount();
  } finally {vi.unstubAllGlobals();}
});
it('narrow library focuses its close button, wraps Tab and restores its opener',async()=>{
  const opener=new TestNode('BUTTON');const root=new TestNode('ROOT');root.children.push(opener);opener.parent=root;
  vi.stubGlobal('document',{activeElement:opener,addEventListener(){},removeEventListener(){}});
  try {
    const pinia=createPinia();useWorkspaceStore(pinia).narrow=true;
    const close=vi.fn();const view=mountView(ReadingSidebar,{open:false,onClose:close},pinia);
    view.updateProps({open:true});await nextTick();await nextTick();
    const first=view.nodes().find(n=>n.props['aria-label']==='Close playlist')!;
    expect(document.activeElement === (first as unknown as Element)).toBe(true);
    const last=view.nodes().filter(n=>n.tagName==='BUTTON').at(-1)!;last.focus();
    last.trigger('keydown',{key:'Tab',preventDefault(){},stopPropagation(){}});await nextTick();
    expect(document.activeElement === (first as unknown as Element)).toBe(true);
    first.trigger('keydown',{key:'Tab',shiftKey:true,preventDefault(){},stopPropagation(){}});await nextTick();
    expect(document.activeElement === (last as unknown as Element)).toBe(true);
    last.trigger('keydown',{key:'Escape',preventDefault(){},stopPropagation(){}});expect(close).toHaveBeenCalledOnce();
    view.updateProps({open:false});await nextTick();expect(document.activeElement === (opener as unknown as Element)).toBe(true);
    view.unmount();
  } finally {vi.unstubAllGlobals();}
});
it('Escape closes project actions and restores their trigger',async()=>{
  const pinia=createPinia();const store=useLibraryStore(pinia);ipc.invoke.mockResolvedValueOnce(emptyLibrary());await store.initialize();
  const view=mountView(LibraryGroup,{group:{id:'p',name:'Team',collapsed:false,matching:0,total:0,items:[]}},pinia);
  const action=view.nodes().find(n=>n.props['aria-label']==='Actions: Team')!;
  action.trigger('click');await nextTick();
  const rename=view.nodes().find(n=>n.tagName==='BUTTON' && n.text==='Rename project')!;
  rename.trigger('keydown',{key:'Escape',stopPropagation(){},preventDefault(){}});await nextTick();
  expect(action.props['aria-expanded']).toBe(false);expect(action.focusCount).toBe(1);view.unmount();
});
it('returns recording rename focus to the persistent row action, not the removed menu item',async()=>{
  vi.stubGlobal('document',{activeElement:null,addEventListener(){},removeEventListener(){}});
  try {
    const pinia=createPinia();const store=useLibraryStore(pinia);
    const data=emptyLibrary();data.recordings=[{id:'a',path:'/a.mp4',display_name:'Original',project_id:null,pinned:false,added_at:0,duration:null}];
    ipc.invoke.mockResolvedValueOnce(data);await store.initialize();
    const view=mountView(LibraryBrowser,{},pinia);
    const action=view.nodes().find(n=>String(n.props['aria-label']).includes(': Original'))!;
    action.focus();action.trigger('click');await nextTick();
    const rename=view.nodes().find(n=>n.tagName==='BUTTON' && n.text==='Rename display name')!;
    rename.focus();rename.trigger('click');await nextTick();
    view.find('name-cancel').trigger('click');await nextTick();
    expect(document.activeElement === (action as unknown as Element)).toBe(true);view.unmount();
  } finally {vi.unstubAllGlobals();}
});
it('returns focus to the invoking control when the name editor closes', async()=>{
  const trigger=new TestNode('BUTTON');const root=new TestNode('ROOT');root.children.push(trigger);trigger.parent=root;
  vi.stubGlobal('document',{activeElement:trigger,addEventListener(){},removeEventListener(){}});
  try {
    const view=mountView(LibraryNameEditor,{initial:'Team',label:'Rename',save:async()=>true},createPinia());
    await nextTick();expect(document.activeElement).toBe(view.find('name-input'));
    view.unmount();await nextTick();expect(document.activeElement).toBe(trigger);
  } finally {vi.unstubAllGlobals();}
});
it('search disables saved group collapse',()=>{
  const pinia=createPinia();const view=mountView(LibraryGroup,{searching:true,group:{id:'p',name:'Team',collapsed:false,matching:1,total:1,items:[]}},pinia);
  const button=view.nodes().find(n=>n.props['aria-expanded']===true)!;
  expect(button.props.disabled).toBe(true);view.unmount();
});
it('recording rename remains mounted when refreshed data moves the row',async()=>{
  const pinia=createPinia();const store=useLibraryStore(pinia);
  const data=emptyLibrary();data.recordings=[{id:'a',path:'/a.mp4',display_name:'Original',project_id:null,pinned:false,added_at:0,duration:null}];
  ipc.invoke.mockResolvedValueOnce(data).mockRejectedValueOnce({kind:'conflict',message:'stale'}).mockResolvedValueOnce({...data,recordings:[]});
  await store.initialize();
  const view=mountView(LibraryBrowser,{},pinia);
  const action=view.nodes().find(n=>String(n.props['aria-label']).includes(': Original'))!;
  (action.props.onClick as ()=>void)();await nextTick();
  const rename=view.nodes().find(n=>n.tagName==='BUTTON' && n.text==='Rename display name')!;
  (rename.props.onClick as ()=>void)();await nextTick();
  await (view.find('name-form').props.onSubmit as (e:unknown)=>Promise<void>)({preventDefault(){}});await nextTick();
  expect(view.find('name-input')).toBeTruthy();expect(store.snapshot.recordings).toEqual([]);view.unmount();
});
it('keeps draft when saving fails and cancels without saving',async()=>{
  const save=vi.fn().mockResolvedValue(false);const cancel=vi.fn();
  const view=mountView(LibraryNameEditor,{initial:'ประชุม',label:'Rename',save,onCancel:cancel},createPinia());
  await nextTick();
  await (view.find('name-form').props.onSubmit as (e:unknown)=>Promise<void>)({preventDefault(){}});
  expect(save).toHaveBeenCalledWith('ประชุม');expect(view.find('name-input')).toBeTruthy();
  (view.find('name-cancel').props.onClick as ()=>void)();expect(cancel).toHaveBeenCalledOnce();view.unmount();
});
it('rejects names longer than 120 Unicode characters',async()=>{
  const save=vi.fn();const view=mountView(LibraryNameEditor,{initial:'🙂'.repeat(121),label:'Rename',save},createPinia());
  await (view.find('name-form').props.onSubmit as (e:unknown)=>Promise<void>)({preventDefault(){}});
  expect(save).not.toHaveBeenCalled();view.unmount();
});
