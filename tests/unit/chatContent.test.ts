import { describe, expect, it } from 'vitest';
import { parseChatText, promptDraft } from '@/utils/chatContent';
import { createSSRApp } from 'vue';
import { createPinia } from 'pinia';
import { renderToString } from 'vue/server-renderer';
import ChatMessage from '@/components/chat/ChatMessage.vue';
describe('safe generated conversation text', () => {
  it('resolves supplied fractional start labels to the exact source start without accepting unrelated stamps', () => {
    for (const source of [{ start: 10.42, end: 12.7 }, { start: 10.92, end: 10.99 }]) {
      const nodes = parseChatText('[00:10] [00:09]', [source], true);
      const parts = nodes.flatMap(node => node.type === 'paragraph' ? node.parts : []);
      expect(parts.filter(part => part.type === 'citation')).toEqual([{ type: 'citation', label: '00:10', seconds: source.start }]);
      expect(JSON.stringify(parts)).toContain('[00:09]');
      expect(JSON.stringify(parseChatText('[00:10]', [source], false))).not.toContain('00:10');
    }
  });
  it('renders model HTML and external links as escaped readable text', async () => {
    const html = await renderToString(
      createSSRApp(ChatMessage, {
        message: {
          id: 'safe',
          role: 'assistant',
          status: 'complete',
          text: '<script>alert(1)</script> [outside](https://example.com)',
        },
      }).use(createPinia()),
    );
    expect(html).toContain('&lt;script&gt;');
    expect(html).not.toContain('<script>');
    expect(html).not.toContain('<a ');
  });
  it('html_and_external_links_are_text', () => {
    const nodes = parseChatText(
      '<script>alert(1)</script>\n\n[link](https://example.com)\n- item',
      [],
      true,
    );
    expect(JSON.stringify(nodes)).toContain('<script>');
    expect(JSON.stringify(nodes)).toContain('https://example.com');
    expect(nodes.map((node) => node.type)).toEqual([
      'paragraph',
      'paragraph',
      'list',
    ]);
    expect(JSON.stringify(nodes)).not.toContain('"href"');
  });
  it('unsupported_citation_not_seekable', () => {
    const text = 'Supported [00:05], unsupported [10:00], malformed [00:65]';
    const nodes = parseChatText(text, [{ start: 0, end: 30 }], true);
    const parts = nodes.flatMap((node) =>
      node.type === 'paragraph' ? node.parts : [],
    );
    expect(parts.filter((part) => part.type === 'citation')).toEqual([
      { type: 'citation', label: '00:05', seconds: 5 },
    ]);
    expect(JSON.stringify(parts)).toContain('[10:00]');
    expect(JSON.stringify(parts)).toContain('[00:65]');
    expect(
      JSON.stringify(parseChatText(text, [{ start: 0, end: 30 }], false)),
    ).not.toContain('"citation"');
  });
  it('code never becomes a seek control and incomplete fences remain readable', () => {
    const nodes = parseChatText(
      '```prompt\nUse [00:05] <script>\n```',
      [{ start: 0, end: 30 }],
      true,
    );
    expect(nodes).toEqual([
      { type: 'code', language: 'prompt', text: 'Use [00:05] <script>' },
    ]);
    expect(parseChatText('```txt\npartial', [], true)).toEqual([
      { type: 'code', language: 'txt', text: 'partial' },
    ]);
  });
  it('two_prompt_blocks_not_auto_applied', () => {
    expect(promptDraft('```prompt\nOne\n```\n```prompt\nTwo\n```')).toBeNull();
    expect(promptDraft('```prompt\nUnfinished')).toBeNull();
    expect(promptDraft('```prompt\n' + 'ก'.repeat(3000) + '\n```')).toBeNull();
    expect(
      promptDraft('Try this:\n```prompt\nKeep only agreed actions\n```'),
    ).toBe('Keep only agreed actions');
  });
});
