import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

// Artifacts and explicit source-comparison notes are necessary, not a truth oracle.
export function assessCompleteParts(run) {
  const fail = reason => ({passed: false, reason});
  if (!/^[a-f0-9]{64}$/.test(run.build ?? '') || !/^[a-f0-9]{64}$/.test(run.source ?? '')) return fail('Missing identity');
  if (!Array.isArray(run.source_clocks) || !run.source_clocks.length) return fail('Missing original clocks');
  const clocks = new Set(run.source_clocks);
  const all = ['custom', 'meeting', 'key_points', 'meeting_detailed'];
  const required = run.required_variants === undefined ? all : run.required_variants;
  if (![JSON.stringify(all), JSON.stringify(all.slice(0, 3))].includes(JSON.stringify(required))) return fail('Invalid acceptance scope');
  for (const name of required) {
    const item = run.cases?.[name];
    if (!item || item.outcome !== 'complete' || item.build !== run.build || item.source !== run.source) return fail(`Incomplete or changed identity: ${name}`);
    if (!Number.isInteger(item.calls) || item.calls < 1 || item.calls > 128) return fail(`Invalid request count: ${name}`);
    const summary = item.summary;
    if (summary?.format !== (name === 'meeting_detailed' ? 'meeting' : name) || summary.meeting_detail !== (name === 'meeting_detailed' ? 'detailed' : 'standard')) return fail(`Wrong metadata: ${name}`);
    const text = summary.markdown?.trim();
    if (!text || /\b[SPOR]\d{6}\b|UNFINISHED RESPONSE|\[\[NO_RELEVANT_CONTENT\]\]/.test(text)) return fail(`Invalid prose: ${name}`);
    const refs = [...text.matchAll(/\[(\d+:\d{2}(?::\d{2})?)\]/g)];
    if (!refs.length || refs.some(([, clock]) => !clocks.has(clock))) return fail(`Unknown source clock: ${name}`);
    if (name === 'custom' && (/\n/.test(text) || /^\s*(?:#|[-*+]\s)/.test(text))) return fail('Custom is not one paragraph');
    const review = item.review;
    if (review?.passed !== true || typeof review.notes !== 'string' || !review.notes.trim() || !Array.isArray(review.source_rows) || !review.source_rows.length || review.source_rows.some(row => !Number.isInteger(row) || row < 1 || row > summary.source_segments)) return fail(`Missing source comparison: ${name}`);
  }
  return {passed: true};
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = assessCompleteParts(JSON.parse(readFileSync(process.argv[2], 'utf8')));
  console.log(JSON.stringify(result, null, 2));
  if (!result.passed) process.exitCode = 1;
}
