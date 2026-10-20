import type { ChatSource } from '@/types/chat';

export type ChatPart =
  | { type: 'text'; text: string }
  | { type: 'citation'; label: string; seconds: number };
export type ChatNode =
  | { type: 'paragraph'; parts: ChatPart[] }
  | { type: 'list'; items: ChatPart[][] }
  | { type: 'code'; language: string; text: string };
export const utf8Length = (text: string): number =>
  new TextEncoder().encode(text).length;

function inline(
  text: string,
  sources: ChatSource[],
  showTimestamps: boolean,
): ChatPart[] {
  const parts: ChatPart[] = [];
  let previous = 0;
  const stamp = /\[(?:(\d{1,4}):)?(\d{1,2}):(\d{2})\]/g;
  for (const match of text.matchAll(stamp)) {
    const seconds =
      Number(match[1] ?? 0) * 3600 + Number(match[2]) * 60 + Number(match[3]);
    const supported =
      Number(match[3]) < 60 &&
      (!match[1] || Number(match[2]) < 60) &&
      sources.some(
        (source) =>
          Number.isFinite(source.start) &&
          Number.isFinite(source.end) &&
          source.start >= 0 &&
          source.end >= source.start &&
          seconds >= source.start &&
          seconds <= source.end,
      );
    if (!supported) continue;
    const index = match.index!;
    if (index > previous)
      parts.push({ type: 'text', text: text.slice(previous, index) });
    if (showTimestamps)
      parts.push({ type: 'citation', label: match[0].slice(1, -1), seconds });
    previous = index + match[0].length;
  }
  if (previous < text.length)
    parts.push({ type: 'text', text: text.slice(previous) });
  return parts;
}

export function parseChatText(
  text: string,
  sources: ChatSource[],
  showTimestamps: boolean,
): ChatNode[] {
  const nodes: ChatNode[] = [];
  let code: { language: string; lines: string[] } | null = null;
  let paragraph: string[] = [];
  const flush = () => {
    if (paragraph.length) {
      nodes.push({
        type: 'paragraph',
        parts: inline(paragraph.join('\n'), sources, showTimestamps),
      });
      paragraph = [];
    }
  };
  for (const line of text.split('\n')) {
    if (code) {
      if (/^\s*```\s*$/.test(line)) {
        nodes.push({
          type: 'code',
          language: code.language,
          text: code.lines.join('\n'),
        });
        code = null;
      } else code.lines.push(line);
      continue;
    }
    const fence = /^\s*```([\w-]*)\s*$/.exec(line);
    if (fence) {
      flush();
      code = { language: fence[1], lines: [] };
      continue;
    }
    if (!line.trim()) {
      flush();
      continue;
    }
    const bullet = /^\s*(?:[-*+] |\d+\. )(.*)$/.exec(line);
    if (bullet) {
      flush();
      const last = nodes[nodes.length - 1];
      const item = inline(bullet[1], sources, showTimestamps);
      if (last?.type === 'list') last.items.push(item);
      else nodes.push({ type: 'list', items: [item] });
    } else paragraph.push(line);
  }
  flush();
  if (code)
    nodes.push({
      type: 'code',
      language: code.language,
      text: code.lines.join('\n'),
    });
  return nodes;
}

export function promptDraft(text: string): string | null {
  let open: { language: string; lines: string[] } | null = null;
  const drafts: string[] = [];
  for (const line of text.split('\n')) {
    if (open) {
      if (/^\s*```\s*$/.test(line)) {
        if (open.language === 'prompt')
          drafts.push(open.lines.join('\n').trim());
        open = null;
      } else open.lines.push(line);
    } else {
      const fence = /^\s*```([\w-]*)\s*$/.exec(line);
      if (fence) open = { language: fence[1].toLowerCase(), lines: [] };
    }
  }
  return !open &&
    drafts.length === 1 &&
    drafts[0].length > 0 &&
    utf8Length(drafts[0]) <= 8192
    ? drafts[0]
    : null;
}
