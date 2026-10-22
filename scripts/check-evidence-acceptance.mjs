import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const required=['requested_choice','person_default','completion_condition','backend_retention','no_inferences','source_times','shape','shared_box','actual_reviewer'];
export function assessRuns(runs) {
  let consecutive=0; let identity=''; let last=0;
  for(const run of runs){
    if(['build','source','settings'].some(key=>typeof run[key]!=='string'||!run[key].trim())||!Number.isInteger(run.round)||run.round<1) throw new Error('Missing build/source/settings/round identity');
    const nextIdentity=JSON.stringify([run.build,run.source,run.settings]);
    if(nextIdentity!==identity||run.round!==last+1) consecutive=0;
    identity=nextIdentity;last=run.round;
    let complete=true;
    for(const mode of ['meeting','key_points','custom']){
      const checks=run.formats?.[mode]?.checks;
      if(!checks){complete=false;continue;}
      if(Object.keys(checks).some(key=>!required.includes(key)))throw new Error('Unknown acceptance criterion');
      const applicable=mode==='custom'?required.filter(key=>!['shared_box','actual_reviewer'].includes(key)):required;
      if(applicable.some(key=>checks[key]!==true))complete=false;
    }
    consecutive=complete?consecutive+1:0;
  }
  return {consecutive,passed:consecutive>=3,node:process.version,icu:process.versions.icu};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  const result=assessRuns(process.argv.slice(2).map(path=>JSON.parse(readFileSync(path,'utf8'))));
  console.log(JSON.stringify(result,null,2));
  if(!result.passed)process.exitCode=1;
}
