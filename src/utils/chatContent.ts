import type { ChatSource } from "@/types/chat";

type ChatLeaf =
  | { type: "text"; text: string }
  | { type: "citation"; label: string; seconds: number };
export type ChatPart = ChatLeaf | { type: "strong"; parts: ChatLeaf[] };
export type ChatNode =
  | { type: "paragraph"; parts: ChatPart[] }
  | { type: "heading"; level: number; parts: ChatPart[] }
  | { type: "list"; items: ChatPart[][] }
  | { type: "code"; language: string; text: string };
export const utf8Length = (text: string): number =>
  new TextEncoder().encode(text).length;

function citations(
  text: string,
  sources: ChatSource[],
  showTimestamps: boolean,
): ChatLeaf[] {
  const parts: ChatLeaf[] = [];
  let previous = 0;
  const stamp = /\[(?:(\d{1,4}):)?(\d{1,2}):(\d{2})\]/g;
  for (const match of text.matchAll(stamp)) {
    const seconds =
      Number(match[1] ?? 0) * 3600 + Number(match[2]) * 60 + Number(match[3]);
    const validSources = sources.filter(
      (source) =>
        Number.isFinite(source.start) &&
        Number.isFinite(source.end) &&
        source.start >= 0 &&
        source.end >= source.start,
    );
    // Rust labels source starts at whole-second precision. A supplied label
    // must still seek the actual fractional start, even for subsecond clips.
    const labelledStart = validSources.find(
      (source) => Math.floor(source.start) === seconds,
    );
    const supported =
      Number(match[3]) < 60 &&
      (!match[1] || Number(match[2]) < 60) &&
      (!!labelledStart ||
        validSources.some(
          (source) =>
            Number.isFinite(source.start) &&
            Number.isFinite(source.end) &&
            source.start >= 0 &&
            source.end >= source.start &&
            seconds >= source.start &&
            seconds <= source.end,
        ));
    if (!supported) continue;
    const index = match.index!;
    if (index > previous)
      parts.push({ type: "text", text: text.slice(previous, index) });
    if (showTimestamps)
      parts.push({
        type: "citation",
        label: match[0].slice(1, -1),
        seconds: labelledStart?.start ?? seconds,
      });
    previous = index + match[0].length;
  }
  if (previous < text.length)
    parts.push({ type: "text", text: text.slice(previous) });
  return parts;
}

function inline(
  text: string,
  sources: ChatSource[],
  showTimestamps: boolean,
): ChatPart[] {
  const parts: ChatPart[] = [];
  let previous = 0;
  // Parse only supported structure, never model-generated HTML. Unclosed
  // markers stay visible while streaming; prompt/code fences bypass this.
  for (const match of text.matchAll(/\*\*(?=\S)([\s\S]*?\S)\*\*/g)) {
    const index = match.index!;
    if (index > previous)
      parts.push(
        ...citations(text.slice(previous, index), sources, showTimestamps),
      );
    parts.push({
      type: "strong",
      parts: citations(match[1], sources, showTimestamps),
    });
    previous = index + match[0].length;
  }
  if (previous < text.length)
    parts.push(...citations(text.slice(previous), sources, showTimestamps));
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
        type: "paragraph",
        parts: inline(paragraph.join("\n"), sources, showTimestamps),
      });
      paragraph = [];
    }
  };
  for (const line of text.split("\n")) {
    if (code) {
      if (/^\s*```\s*$/.test(line)) {
        nodes.push({
          type: "code",
          language: code.language,
          text: code.lines.join("\n"),
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
    const heading = /^ {0,3}(#{1,6})[\t ]+(.+?)(?:[\t ]+#+)?[\t ]*$/.exec(line);
    if (heading) {
      flush();
      nodes.push({
        type: "heading",
        level: heading[1].length,
        parts: inline(heading[2], sources, showTimestamps),
      });
      continue;
    }
    const bullet = /^\s*(?:[-*+] |\d+\. )(.*)$/.exec(line);
    if (bullet) {
      flush();
      const last = nodes[nodes.length - 1];
      const item = inline(bullet[1], sources, showTimestamps);
      if (last?.type === "list") last.items.push(item);
      else nodes.push({ type: "list", items: [item] });
    } else paragraph.push(line);
  }
  flush();
  if (code)
    nodes.push({
      type: "code",
      language: code.language,
      text: code.lines.join("\n"),
    });
  return nodes;
}

export function promptDraft(text: string): string | null {
  let open: { language: string; lines: string[] } | null = null;
  const drafts: string[] = [];
  for (const line of text.split("\n")) {
    if (open) {
      if (/^\s*```\s*$/.test(line)) {
        if (open.language === "prompt")
          drafts.push(open.lines.join("\n").trim());
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
