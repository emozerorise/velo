import {test} from 'node:test';
import assert from 'node:assert/strict';
import {assessRuns} from '../../scripts/check-evidence-acceptance.mjs';
const checks={requested_choice:true,person_default:true,completion_condition:true,backend_retention:true,no_inferences:true,source_times:true,shape:true,shared_box:true,actual_reviewer:true};
const round=(n)=>({round:n,build:'same-code-digest',source:'same-source-digest',settings:'same-settings-digest',formats:{meeting:{checks:{...checks}},key_points:{checks:{...checks}},custom:{checks:{...checks}}}});
test('three complete same-build rounds pass',()=>assert.equal(assessRuns([round(1),round(2),round(3)]).passed,true));
test('any failed format resets the consecutive sequence',()=>{const failed=round(2);failed.formats.custom.checks.completion_condition=false;assert.equal(assessRuns([round(1),failed,round(3)]).consecutive,1);});
test('missing formats/checks or unknown check names cannot certify a round',()=>{
 const missing=round(1);delete missing.formats.custom;assert.equal(assessRuns([missing]).consecutive,0);
 const unknown=round(1);unknown.formats.meeting.checks.magic_accuracy=true;assert.throws(()=>assessRuns([unknown]));
});
test('different code builds cannot be assembled into a pass',()=>{const changed=round(3);changed.build='changed-code';assert.equal(assessRuns([round(1),round(2),changed]).consecutive,1);});
test('different source or settings identities reset the sequence',()=>{
 for(const key of ['source','settings']){
  const changed=round(3);changed[key]='changed';
  assert.equal(assessRuns([round(1),round(2),changed]).consecutive,1);
 }
});
test('missing and blank identities cannot certify old records',()=>{
 for(const key of ['build','source','settings']){
  for(const value of [undefined,'','   ']){
   const bad=round(1);bad[key]=value;assert.throws(()=>assessRuns([bad]));
  }
 }
 for(const n of [0,-1,1.5])assert.throws(()=>assessRuns([round(n)]));
});
test('gaps and reordered rounds cannot certify a continuous run',()=>{
 assert.equal(assessRuns([round(1),round(3),round(4)]).consecutive,2);
 assert.equal(assessRuns([round(1),round(2),round(2)]).consecutive,1);
});
test('optional custom-only secondary criteria need not be supplied',()=>{
 const runs=[round(1),round(2),round(3)];
 for(const run of runs){delete run.formats.custom.checks.shared_box;delete run.formats.custom.checks.actual_reviewer;}
 assert.equal(assessRuns(runs).passed,true);
});
