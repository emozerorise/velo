/** Turns the model's Markdown into nodes the panel renders as real elements.
 *
 *  Only the small subset the prompt asks for is understood -- headings,
 *  bullets, bold, and `[mm:ss]` citations. Rendering nodes instead of HTML
 *  keeps model output out of `v-html`, and the citations become the buttons
 *  that make a summary a way to navigate the recording. */

export type InlinePart =
  | { type: 'text'; text: string }
  | { type: 'bold'; text: string }
  | { type: 'stamp'; label: string; seconds: number };

export type SummaryNode =
  | { type: 'heading'; text: string }
  | { type: 'bullet'; parts: InlinePart[]; depth: number }
  | { type: 'paragraph'; parts: InlinePart[] };

export type SummarySection = {
  title: string | null;
  kind: 'overview' | 'decisions' | 'actions' | 'topics' | 'questions' | 'other';
  nodes: Exclude<SummaryNode, { type: 'heading' }>[];
};

export function groupSummarySections(markdown: string): SummarySection[] {
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
  for (const node of parseSummary(markdown)) {
    if (node.type === 'heading') {
      const heading = node.text.toLowerCase();
      sections.push({
        title: node.text,
        kind: Object.prototype.hasOwnProperty.call(kinds, heading) ? kinds[heading] : 'other',
        nodes: [],
      });
    } else {
      if (!sections.length) sections.push({ title: null, kind: 'other', nodes: [] });
      sections[sections.length - 1].nodes.push(node);
    }
  }
  // Only presentation changes: copying and caching keep the model's original Markdown.
  const priority = (section: SummarySection): number => {
    if (section.title === null) return -1;
    return { overview: 0, decisions: 1, actions: 2, topics: 3, questions: 4, other: 5 }[
      section.kind
    ];
  };
  return sections.sort((a, b) => priority(a) - priority(b));
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
    if (at > last) parts.push({ type: 'text', text: text.slice(last, at) });
    parts.push({ type: 'bold', text: match[1] });
    last = at + match[0].length;
  }

  if (last < text.length) parts.push({ type: 'text', text: text.slice(last) });
  return parts;
}

export function parseInline(line: string): InlinePart[] {
  const parts: InlinePart[] = [];
  let last = 0;

  for (const match of line.matchAll(STAMP)) {
    const at = match.index ?? 0;
    if (at > last) parts.push(...parseBold(line.slice(last, at)));
    parts.push({
      type: 'stamp',
      label: match[0].slice(1, -1),
      seconds: stampToSeconds(match[1], match[2], match[3]),
    });
    last = at + match[0].length;
  }

  if (last < line.length) parts.push(...parseBold(line.slice(last)));
  return parts;
}

export function parseSummary(markdown: string): SummaryNode[] {
  const nodes: SummaryNode[] = [];

  for (const raw of markdown.split('\n')) {
    const line = raw.trimEnd();
    if (line.trim() === '') continue;

    const heading = /^#{1,6}\s+(.*)$/.exec(line.trim());
    if (heading) {
      nodes.push({
        type: 'heading',
        text: heading[1].replace(/\*\*/g, '').trim(),
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
        parts: parseInline(body),
        depth: Math.min(Math.floor(bullet[1].length / 2), 2),
      });
      continue;
    }

    nodes.push({ type: 'paragraph', parts: parseInline(line.trim()) });
  }

  return nodes;
}
