import {beforeEach,describe,it,expect,vi} from 'vitest';
import {createPinia,setActivePinia,getActivePinia} from 'pinia';
import {createSSRApp} from 'vue';
import {renderToString} from 'vue/server-renderer';
import SummaryTab from '@/components/transcript/SummaryTab.vue';
import {useSummaryStore} from '@/stores/summaryStore';
import {usePlayerStore} from '@/stores/playerStore';
import {setLocale} from '@/composables/useI18n';
import {parseInline,parseSummary,summaryForCopy} from '@/utils/summaryMarkdown';
const invoke=vi.fn();
vi.mock('@tauri-apps/api/core',()=>({invoke:(...a:unknown[])=>invoke(...a)}));
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn().mockResolvedValue(()=>{})}));
describe('evidence summary status',()=>{
 beforeEach(()=>{setActivePinia(createPinia());setLocale('en');invoke.mockReset();usePlayerStore().mediaInfo={path:'/clip.wav',file_name:'clip.wav',duration:30,video_codec:null,audio_codec:'pcm',width:0,height:0,fps:0,hwdec_current:null};useSummaryStore().summary={path:'/clip.wav',model:'old',language:'en',markdown:'Previous content',created_at:1,source_segments:1};});
 const html=()=>renderToString(createSSRApp(SummaryTab).use(getActivePinia()!));
 it.each([undefined,null,0])('does not warn about absent/empty omission metadata (%s)',async(value)=>{
  Object.assign(useSummaryStore().summary!,{omitted_uncertain_claims:value});
  expect(await html()).not.toContain('data-testid="summary-omission"');
 });
 it.each([
  ['writing_summary','AI is writing your summary'],
  ['checking_references','Checking timestamp references'],
  ['extracting_evidence','Extracting source evidence'],
  ['checking_evidence','Checking evidence and conditions'],
  ['composing_summary','Arranging supported points'],
 ] as const)('shows %s as step four and routes Cancel',async(stage,label)=>{
  const store=useSummaryStore();
  // New backend stages arrive over the existing open-ended event boundary.
  Object.assign(store,{stage,chained:true});
  expect(await html()).toContain(label);expect(store.step).toBe(4);
  await store.cancel();expect(invoke).toHaveBeenCalledWith('summary_cancel');
 });
 it.each(['en','th'])('shows a partial-uncertainty notice, not certification (%s)',async(locale)=>{
  setLocale(locale);const store=useSummaryStore();Object.assign(store.summary!,{omitted_uncertain_claims:2});
  expect(await html()).toContain(locale==='th'?'ละรายละเอียด 2 ประเด็น':'Omitted 2 details');
  Object.assign(store.summary!,{omitted_uncertain_claims:0});expect(await html()).not.toContain('data-testid="summary-omission"');
 });
 it('retains the old document and explains unsupported custom layouts',async()=>{
  useSummaryStore().error='unsupported_summary_layout';const result=await html();expect(result).toContain('Previous content');expect(result).toContain('This layout needs rewriting');
 });
 it('escaped_source_stamp_is_text_not_navigation',()=>{
  const sources=[{start:10.25,end:12.75,text:'literal'},{start:12.75,end:15,text:'source'}];
  expect(parseInline('\\[00:10] [00:12]',sources)).toEqual([{type:'text',text:'[00:10] '},{type:'stamp',label:'00:12',seconds:12.75}]);
  expect(summaryForCopy('Literal \\[00:10] [00:12]',false)).toBe('Literal \\[00:10]');
  expect(parseSummary('\\## literal\n\\- literal\n\\*\\*not bold\\*\\*',sources).every(n=>n.type==='paragraph')).toBe(true);
 });
});
