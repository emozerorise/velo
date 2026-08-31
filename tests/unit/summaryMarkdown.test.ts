import { describe, it, expect } from 'vitest';
import { parseInline, parseSummary } from '@/utils/summaryMarkdown';

describe('parseInline', () => {
  it('turns [mm:ss] into a seekable stamp', () => {
    const parts = parseInline('เลื่อน deploy ไปศุกร์ [12:34]');
    const stamp = parts.find((p) => p.type === 'stamp');

    expect(stamp).toEqual({ type: 'stamp', label: '12:34', seconds: 754 });
  });

  it('understands the hour form', () => {
    const parts = parseInline('[1:02:03] ต่อวาระถัดไป');
    expect(parts[0]).toEqual({ type: 'stamp', label: '1:02:03', seconds: 3723 });
  });

  it('keeps the text around a stamp', () => {
    const parts = parseInline('ก่อน [00:05] หลัง');

    expect(parts.map((p) => ('text' in p ? p.text : p.label))).toEqual(['ก่อน ', '00:05', ' หลัง']);
  });

  it('reads bold segments', () => {
    const parts = parseInline('**ฝน**: แจ้ง QA');
    expect(parts[0]).toEqual({ type: 'bold', text: 'ฝน' });
  });

  it('leaves text without a stamp alone', () => {
    const parts = parseInline('ไม่มีเวลากำกับ');
    expect(parts).toEqual([{ type: 'text', text: 'ไม่มีเวลากำกับ' }]);
  });

  it('ignores a bracket that is not a timestamp', () => {
    const parts = parseInline('ดูที่ [เอกสาร] ประกอบ');
    expect(parts.some((p) => p.type === 'stamp')).toBe(false);
  });
});

describe('parseSummary', () => {
  const markdown = [
    '## ภาพรวม',
    'ทีมคุยเรื่องกำหนดปล่อยรุ่น',
    '',
    '## สิ่งที่ตัดสินใจ',
    '- เลื่อน deploy ไปศุกร์ [12:34]',
    '  - แจ้งลูกค้าด้วย [12:50]',
    '',
    '## เรื่องที่ยังค้าง',
    '-',
  ].join('\n');

  it('separates headings, paragraphs and bullets', () => {
    const nodes = parseSummary(markdown);

    expect(nodes.filter((n) => n.type === 'heading')).toHaveLength(3);
    expect(nodes.filter((n) => n.type === 'bullet')).toHaveLength(2);
    expect(nodes.filter((n) => n.type === 'paragraph')).toHaveLength(1);
  });

  it('keeps nesting depth', () => {
    const bullets = parseSummary(markdown).filter((n) => n.type === 'bullet');
    expect(bullets.map((b) => (b.type === 'bullet' ? b.depth : -1))).toEqual([0, 1]);
  });

  it('drops the placeholder used for an empty section', () => {
    const nodes = parseSummary('## ค้าง\n-\n');
    expect(nodes).toHaveLength(1);
    expect(nodes[0].type).toBe('heading');
  });

  it('survives a model that ignored the format entirely', () => {
    const nodes = parseSummary('ประชุมสั้นมาก ไม่มีอะไรต้องทำต่อ');
    expect(nodes).toEqual([
      { type: 'paragraph', parts: [{ type: 'text', text: 'ประชุมสั้นมาก ไม่มีอะไรต้องทำต่อ' }] },
    ]);
  });
});
