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
