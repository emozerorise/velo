import {test} from 'node:test';
import assert from 'node:assert/strict';
import {assessCompleteParts} from '../../scripts/check-summary-parts-acceptance.mjs';
function run() {
  const result = {build: 'a'.repeat(64), source: 'b'.repeat(64), source_clocks: ['00:10', '1:01:00'], cases: {}};
  for (const name of ['custom', 'meeting', 'key_points', 'meeting_detailed']) result.cases[name] = {outcome: 'complete', build: result.build, source: result.source, calls: 20, summary: {format: name === 'meeting_detailed' ? 'meeting' : name, meeting_detail: name === 'meeting_detailed' ? 'detailed' : 'standard', markdown: 'Supported [00:10] [1:01:00]', source_segments: 32}, review: {passed: true, notes: 'Compared adjacent meaning and distant condition to original quoted rows.', source_rows: [1, 32]}};
  return result;
}
test('all same-build complete variants with explicit review pass', () => assert.equal(assessCompleteParts(run()).passed, true));
test('only the explicitly selected three-format scope can omit detailed meeting', () => {
  const selected = run(); selected.required_variants = ['custom', 'meeting', 'key_points']; delete selected.cases.meeting_detailed;
  assert.equal(assessCompleteParts(selected).passed, true);
  selected.required_variants = ['custom', 'meeting'];
  assert.equal(assessCompleteParts(selected).passed, false);
});
test('corrupt or incomplete artifacts cannot certify acceptance', () => {
  for (const corrupt of [
    r => delete r.cases.custom,
    r => r.cases.custom.summary.markdown += ' [01:23]',
    r => r.cases.custom.summary.markdown += ' [S000001]',
    r => r.cases.meeting.summary.format = 'custom',
    r => r.cases.custom.summary.markdown += '\nOther [00:10]',
    r => r.cases.meeting.outcome = 'failed',
    r => r.cases.meeting.build = 'c'.repeat(64),
    r => r.cases.meeting.source = 'c'.repeat(64),
    r => delete r.cases.meeting.review,
    r => r.cases.meeting.review.notes = '',
    r => r.cases.meeting.calls = 129,
    r => r.cases.meeting.summary.markdown = 'UNFINISHED RESPONSE [00:10]',
  ]) {const bad = run(); corrupt(bad); assert.equal(assessCompleteParts(bad).passed, false);}
});
