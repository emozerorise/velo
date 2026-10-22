import { expect, it } from 'vitest';
import { emptyLibrary, type LibraryRecording } from '@/types/library';
import { visibleLibrary, recordingName } from '@/utils/libraryView';
const entry=(id:string):LibraryRecording=>({id,path:`/source/${id}.mp4`,display_name:null,project_id:'p',pinned:false,added_at:Number(id),duration:null});
it('search reveals collapsed groups without persisting expansion or duplicating pinned counts',()=>{
  const data=emptyLibrary();data.projects=[{id:'p',name:'Team',created_at:0}];data.recordings=[{...entry('1'),display_name:'ประชุม',pinned:true},entry('2')];data.preferences.collapsed_project_ids=['p'];
  const result=visibleLibrary(data,'ประชุม',[]);
  expect(result.groups.find(g=>g.id==='p')?.collapsed).toBe(false);expect(result.groups.find(g=>g.id==='p')?.matching).toBe(1);
  expect(result.pinned).toHaveLength(1);expect(data.recordings).toHaveLength(2);expect(data.preferences.collapsed_project_ids).toEqual(['p']);
  expect(visibleLibrary(data,'',[]).groups.find(g=>g.id==='p')?.collapsed).toBe(true);
  expect(visibleLibrary(data,'1.mp4',[]).pinned).toHaveLength(1);
});
it('sorts without modifying source order and handles 1000 records',()=>{
  const data=emptyLibrary();data.recordings=Array.from({length:1000},(_,i)=>({...entry(String(i)),project_id:null}));
  expect(visibleLibrary(data,'',[]).groups[0].items[0].id).toBe('999');
  expect(data.recordings[0].id).toBe('0');
  data.preferences.sort='played';
  expect(visibleLibrary(data,'',[{path:'/source/7.mp4',file_name:'7.mp4',last_position:0,duration:20,last_played_timestamp:100}]).groups[0].items[0].id).toBe('7');
  expect(recordingName({...entry('1'),display_name:'ใหม่'})).toBe('ใหม่');
});
