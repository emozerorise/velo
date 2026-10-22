/** Turns the model's Markdown into nodes the panel renders as real elements.
 *
 *  Only the small subset the prompt asks for is understood -- headings,
 *  bullets, bold, and `[mm:ss]` citations. Rendering nodes instead of HTML
 *  keeps model output out of `v-html`, and the citations become the buttons
 *  that make a summary a way to navigate the recording. */

import type { SummaryFormat } from '@/types/settings';
import type { TranscriptSegment } from '@/types/transcript';

export type InlinePart =
  | { type: 'text'; text: string }
  | { type: 'bold'; text: string }
  | { type: 'stamp'; label: string; seconds: number };

export type SummaryNode =
  | { type: 'heading'; text: string; level: number }
  | { type: 'bullet'; parts: InlinePart[]; depth: number }
  | { type: 'paragraph'; parts: InlinePart[] };

export type SummarySection = {
  title: string | null;
  kind: 'overview' | 'decisions' | 'actions' | 'topics' | 'questions' | 'other';
  nodes: (
    | Exclude<SummaryNode, { type: 'heading' }>
    | { type: 'subheading'; text: string; depth: number }
  )[];
};

export function summaryForCopy(
  markdown: string,
  showTimestamps: boolean,
): string {
  return showTimestamps
    ? markdown
    : markdown.replace(new RegExp(`[ \\t]*${STAMP.source}`, 'g'), (match, ...args: unknown[]) => {
        const start = args[args.length - 2] as number;
        return escapedAt(markdown, start + match.indexOf('[')) ? match : '';
      });
}

function escapedAt(text: string, at: number): boolean {
  let slashes = 0;
  for (let i = at - 1; i >= 0 && text[i] === '\\'; i--) slashes++;
  return slashes % 2 === 1;
}

function literalText(text: string): string {
  return text.replace(/\\([\\[\]*_#\-+`>.])/g, '$1');
}

export function groupSummarySections(
  markdown: string,
  format: SummaryFormat = 'meeting',
  sources?: TranscriptSegment[],
): SummarySection[] {
  const kinds: Record<string, SummarySection['kind']> = {
    overview: 'overview',
    ภาพรวม: 'overview',
    decisions: 'decisions',
    สิ่งที่ตัดสินใจ: 'decisions',
    'action items': 'actions',
    งานที่ต้องทำ: 'actions',
    'topics discussed': 'topics',
    ประเด็นที่คุยกัน: 'topics',
    'open questions': 'questions',
    เรื่องที่ยังค้าง: 'questions',
  };
  const sections: SummarySection[] = [];
  let sectionLevel = 0;
  for (const node of parseSummary(markdown, sources)) {
    if (node.type === 'heading') {
      // Outcome sorting must not detach nested task labels from their parent.
      if (sectionLevel > 0 && node.level > sectionLevel) {
        sections[sections.length - 1].nodes.push({
          type: 'subheading',
          text: node.text,
          depth: node.level - sectionLevel,
        });
        continue;
      }
      sectionLevel = node.level;
      const heading = node.text.toLowerCase();
      sections.push({
        title: node.text,
        kind: Object.prototype.hasOwnProperty.call(kinds, heading)
          ? kinds[heading]
          : 'other',
        nodes: [],
      });
    } else {
      if (!sections.length)
        sections.push({ title: null, kind: 'other', nodes: [] });
      sections[sections.length - 1].nodes.push(node);
    }
  }
  // Only presentation changes: copying and caching keep the model's original Markdown.
  const priority = (section: SummarySection): number => {
    if (section.title === null) return -1;
    return {
      overview: 0,
      decisions: 1,
      actions: 2,
      topics: 3,
      questions: 4,
      other: 5,
    }[section.kind];
  };
  return format === 'meeting'
    ? sections.sort((a, b) => priority(a) - priority(b))
    : sections;
}

/** `[mm:ss]` or `[h:mm:ss]`, the two shapes the prompt asks for. */
const STAMP = /\[(?:(\d{1,2}):)?(\d{1,2}):(\d{2})\]/g;
const BOLD = /\*\*(.+?)\*\*/g;

export function stampToSeconds(
  hours: string | undefined,
  minutes: string,
  seconds: string,
): number {
  return Number(hours ?? 0) * 3600 + Number(minutes) * 60 + Number(seconds);
}

function parseBold(text: string): InlinePart[] {
  const parts: InlinePart[] = [];
  let last = 0;

  for (const match of text.matchAll(BOLD)) {
    const at = match.index ?? 0;
    if (escapedAt(text, at) || escapedAt(text, at + match[0].length - 2)) continue;
    if (at > last) parts.push({ type: 'text', text: literalText(text.slice(last, at)) });
    parts.push({ type: 'bold', text: literalText(match[1]) });
    last = at + match[0].length;
  }

  if (last < text.length) parts.push({ type: 'text', text: literalText(text.slice(last)) });
  return parts;
}

export function parseInline(
  line: string,
  sources?: TranscriptSegment[],
): InlinePart[] {
  const parts: InlinePart[] = [];
  let last = 0;

  for (const match of line.matchAll(STAMP)) {
    const at = match.index ?? 0;
    if (escapedAt(line, at)) continue;
    if (at > last) parts.push(...parseBold(line.slice(last, at)));
    const seconds = stampToSeconds(match[1], match[2], match[3]);
    const canonical =
      seconds >= 3600
        ? `${Math.floor(seconds / 3600)}:${String(Math.floor(seconds / 60) % 60).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`
        : `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
    const matches = sources?.filter(
      (segment) =>
        Number.isFinite(segment.start) &&
        Number.isFinite(segment.end) &&
        segment.start >= 0 &&
        segment.end >= segment.start &&
        Math.floor(segment.start) === seconds,
    );
    const source = matches?.length === 1 ? matches[0] : undefined;
    // Never snap a legacy/invented label to the nearest segment or interval.
    // Undefined sources retain the standalone parser API; the reader always supplies a list.
    if (sources === undefined || (match[0] === `[${canonical}]` && source)) {
      parts.push({
        type: 'stamp',
        label: match[0].slice(1, -1),
        seconds: source?.start ?? seconds,
      });
    } else {
      parts.push({ type: 'text', text: match[0] });
    }
    last = at + match[0].length;
  }

  if (last < line.length) parts.push(...parseBold(line.slice(last)));
  return parts;
}

export function parseSummary(
  markdown: string,
  sources?: TranscriptSegment[],
): SummaryNode[] {
  const nodes: SummaryNode[] = [];

  for (const raw of markdown.split('\n')) {
    const line = raw.trimEnd();
    if (line.trim() === '') continue;

    const heading = /^(#{1,6})\s+(.*)$/.exec(line.trim());
    if (heading) {
      nodes.push({
        type: 'heading',
        text: heading[2].replace(/\*\*/g, '').trim(),
        level: heading[1].length,
      });
      continue;
    }

    // A lone "-" is how the prompt asks for an empty section.
    if (line.trim() === '-') continue;

    // Nested bullets keep their indentation, so sub-points stay sub-points.
    const bullet = /^(\s*)(?:[-*+]|\d+\.)\s+(.*)$/.exec(line);
    if (bullet) {
      const body = bullet[2].trim();
      if (body === '') continue;
      nodes.push({
        type: 'bullet',
        parts: parseInline(body, sources),
        depth: Math.min(Math.floor(bullet[1].length / 2), 2),
      });
      continue;
    }

    nodes.push({ type: 'paragraph', parts: parseInline(line.trim(), sources) });
  }

  return nodes;
}
