import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, getActivePinia, setActivePinia } from "pinia";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import SummaryTab from "@/components/transcript/SummaryTab.vue";
import SummaryLine from "@/components/transcript/SummaryLine.vue";
import SummarySectionBody from "@/components/transcript/SummarySectionBody.vue";
import {
  groupSummarySections,
  parseInline,
  parseSummary,
} from "@/utils/summaryMarkdown";
import { usePlayerStore } from "@/stores/playerStore";
import { useSummaryStore } from "@/stores/summaryStore";
import { setLocale } from "@/composables/useI18n";
import { useSettingsStore } from "@/stores/settingsStore";

describe("summary reading sections", () => {
  it("preserves source section order for detailed meetings without changing standard meeting order", () => {
    const markdown =
      "## Overview\nOverview\n## Topics discussed\nReason\n## Decisions\nDecision";
    expect(
      groupSummarySections(markdown, "meeting", [], "detailed").map(
        (s) => s.kind,
      ),
    ).toEqual(["overview", "topics", "decisions"]);
    expect(
      groupSummarySections(markdown, "meeting", [], "standard").map(
        (s) => s.kind,
      ),
    ).toEqual(["overview", "decisions", "topics"]);
  });
  it("keeps model subheadings and their cited tasks inside the parent section", async () => {
    const sections = groupSummarySections(
      "## Action items\n### Agreed tasks\n- Task [00:30]\n### Suggestions\n- Proposal [00:37]\n## Overview\nIntro",
    );
    expect(sections.map((s) => s.title)).toEqual(["Overview", "Action items"]);
    const tasks = sections[1];
    expect(tasks.nodes).toHaveLength(4);
    const html = await renderToString(
      createSSRApp(SummarySectionBody, { nodes: tasks.nodes }),
    );
    expect(html).toContain("<h4");
    expect(html).toContain("Agreed tasks");
    expect(html).toContain("Task");
    expect(html.indexOf("Task")).toBeLessThan(html.indexOf("Suggestions"));
    expect(html).not.toContain("Not specified");
  });
  it("preserves the requested section order for custom and key-point summaries", () => {
    for (const format of ["custom", "key_points"] as const) {
      const sections = groupSummarySections(
        "## My focus\nOriginal\n## Action items\n- Task\n## Overview\nLast",
        format,
      );
      expect(sections.map((section) => section.title)).toEqual([
        "My focus",
        "Action items",
        "Overview",
      ]);
    }
  });
  it("treats headings matching JavaScript property names as ordinary model text", () => {
    expect(groupSummarySections("## constructor\nText")[0].kind).toBe("other");
  });
  it.each([
    [
      "Overview",
      "Topics discussed",
      "Decisions",
      "Action items",
      "Open questions",
    ],
    [
      "ภาพรวม",
      "ประเด็นที่คุยกัน",
      "สิ่งที่ตัดสินใจ",
      "งานที่ต้องทำ",
      "เรื่องที่ยังค้าง",
    ],
  ])(
    "prioritises outcomes without losing the original blocks (%s)",
    (...titles) => {
      const markdown = titles
        .map((title, i) => `## ${title}\n- Point ${i} [00:10]\n  - Detail ${i}`)
        .join("\n");
      const sections = groupSummarySections(markdown);
      expect(sections.map((section) => section.kind)).toEqual([
        "overview",
        "decisions",
        "actions",
        "topics",
        "questions",
      ]);
      expect(sections.flatMap((section) => section.nodes)).toHaveLength(10);
      expect(sections[1].nodes).toEqual(
        parseSummary("- Point 2 [00:10]\n  - Detail 2"),
      );
    },
  );

  it("keeps preambles, custom headings, duplicate headings and empty sections", () => {
    const sections = groupSummarySections(
      "Preamble\n## Custom\nText\n## Custom\n- Other\n## Decisions\n-",
    );
    expect(sections.map(({ title }) => title)).toEqual([
      null,
      "Decisions",
      "Custom",
      "Custom",
    ]);
    expect(sections.flatMap(({ nodes }) => nodes)).toEqual([
      ...parseSummary("Preamble"),
      ...parseSummary("Text"),
      ...parseSummary("- Other"),
    ]);
    expect(sections[1].nodes).toEqual([]);
    expect(groupSummarySections("")).toEqual([]);
  });
});

describe("summary reading presentation", () => {
  it.each([false, true])(
    "uses saved detail for its badge and initially expanded discussion, not next-run settings (%s)",
    async (detailed) => {
      setLocale("en");
      const store = useSummaryStore();
      store.summary = {
        ...store.summary!,
        meeting_detail: detailed ? "detailed" : "standard",
      };
      useSettingsStore().settings.summary.meeting_detail = detailed
        ? "standard"
        : "detailed";
      const html = await renderToString(
        createSSRApp(SummaryTab).use(getActivePinia()!),
      );
      expect(/<details\b[^>]*\bopen(?:\s|>|=)/.test(html)).toBe(detailed);
      expect(html.includes('data-testid="saved-summary-detail"')).toBe(
        detailed,
      );
      if (detailed)
        expect(html.indexOf("Context")).toBeLessThan(html.indexOf("Ship it"));
      else
        expect(html.indexOf("Ship it")).toBeLessThan(html.indexOf("Context"));
    },
  );
  it("explains intermediate grouping while a long summary is running", async () => {
    setLocale("th");
    const store = useSummaryStore();
    store.stage = "reducing";
    store.mapped = 1;
    store.mapTotal = 4;
    const html = await renderToString(
      createSSRApp(SummaryTab).use(getActivePinia()!),
    );
    expect(html).toContain("กำลังย่อโน้ตกลุ่มที่ 2/4");
    expect(store.progress).toBe(0.25);
  });
  it.each(["en", "th"])(
    "labels a failed regeneration as the previous saved summary (%s)",
    async (locale) => {
      setLocale(locale);
      const store = useSummaryStore();
      const original = store.summary!.markdown;
      store.error = "summary_output_limit";
      const html = await renderToString(
        createSSRApp(SummaryTab).use(getActivePinia()!),
      );
      expect(html).toContain(
        locale === "th"
          ? "สรุปใหม่ไม่สำเร็จ กำลังแสดงฉบับเดิม"
          : "Could not regenerate. Showing the previous summary.",
      );
      expect(html).toContain(locale === "th" ? "ขีดจำกัด" : "output limit");
      expect(html).not.toContain("summary_output_limit");
      expect(html).toContain("Ship it");
      expect(store.summary!.markdown).toBe(original);
    },
  );
  it("does not claim a previous summary exists on a first-run failure", async () => {
    setLocale("th");
    const store = useSummaryStore();
    store.summary = null;
    store.error = "summary_output_limit";
    const html = await renderToString(
      createSSRApp(SummaryTab).use(getActivePinia()!),
    );
    expect(html).toContain("ขีดจำกัด");
    expect(html).not.toContain("กำลังแสดงฉบับเดิม");
    expect(html).not.toContain("summary_output_limit");
  });
  it("localises an interrupted stream without presenting it as a new result", async () => {
    setLocale("th");
    useSummaryStore().error = "The provider stream ended before completion";
    const html = await renderToString(
      createSSRApp(SummaryTab).use(getActivePinia()!),
    );
    expect(html).toContain("การเชื่อมต่อสิ้นสุดก่อน AI ตอบครบ");
    expect(html).toContain("กำลังแสดงฉบับเดิม");
    expect(html).not.toContain("ended before completion");
  });
  it.each(["en", "th"])(
    "keeps a topics-only extractive summary visible on first render (%s)",
    async (locale) => {
      setLocale(locale);
      const store = useSummaryStore();
      store.summary = {
        ...store.summary!,
        format: "meeting",
        markdown:
          locale === "th"
            ? "## ประเด็นที่คุยกัน\n- “เมื่อตรวจเสร็จ อาจไม่แสดงใช่ไหม” [00:10]"
            : "## Topics discussed\n- “Only after completion, could it disappear?” [00:10]",
      };
      const original = store.summary.markdown;
      const html = await renderToString(
        createSSRApp(SummaryTab).use(getActivePinia()!),
      );
      expect(html).not.toContain("<details");
      expect(html).toContain(
        locale === "th" ? "เมื่อตรวจเสร็จ" : "Only after completion",
      );
      expect(store.summary.markdown).toBe(original);
      expect(store.isRunning).toBe(false);
    },
  );
  it("shows a validation failure alongside the unchanged previous summary", async () => {
    useSummaryStore().error =
      "Summary validation failed; previous summary unchanged";
    const html = await renderToString(
      createSSRApp(SummaryTab).use(getActivePinia()!),
    );
    expect(html).toContain('role="alert"');
    expect(html).toContain("Summary validation failed");
    expect(html).toContain("Ship it");
  });
  it("uses the saved summary format, not the next-run preference, without collapsing custom content", async () => {
    setLocale("en");
    useSummaryStore().summary = {
      ...useSummaryStore().summary!,
      format: "custom",
      markdown:
        "## My focus\nStart here\n## Topics discussed\nKeep visible\n## Overview\nLast",
    };
    useSettingsStore().settings.summary.format = "meeting";
    const html = await renderToString(
      createSSRApp(SummaryTab).use(getActivePinia()!),
    );
    expect(html).not.toContain("<details");
    expect(html.indexOf("Start here")).toBeLessThan(
      html.indexOf("Keep visible"),
    );
    expect(html.indexOf("Keep visible")).toBeLessThan(html.indexOf("Last"));
  });
  beforeEach(() => {
    setActivePinia(createPinia());
    usePlayerStore().mediaInfo = {
      path: "/sample.mp3",
      file_name: "sample.mp3",
      duration: 60,
      video_codec: null,
      audio_codec: "mp3",
      width: 0,
      height: 0,
      fps: 0,
      hwdec_current: null,
    };
    useSummaryStore().summary = {
      path: "/sample.mp3",
      model: "test-model",
      language: "en",
      created_at: 0,
      source_segments: 0,
      markdown:
        "## Overview\nA short overview.\n## Topics discussed\n- Context [00:10]\n## Decisions\n- Ship it [00:20]\n## Action items\n- **Owner**: follow up [00:30]",
    };
  });

  it("hides citation controls without changing the cached summary", async () => {
    const settings = useSettingsStore();
    settings.settings.general = {
      ...settings.settings.general,
      show_summary_timestamps: false,
    };
    const original = useSummaryStore().summary?.markdown;
    const html = await renderToString(
      createSSRApp(SummaryTab).use(getActivePinia()!),
    );
    expect(html).not.toContain("Jump to 00:20");
    expect(html).not.toContain("ข้ามไปที่ 00:20");
    expect(html).toContain("Ship it");
    expect(useSummaryStore().summary?.markdown).toBe(original);
  });

  it.each(["en", "th"])(
    "offers timestamp and prompt controls (%s)",
    async (locale) => {
      setLocale(locale);
      const html = await renderToString(
        createSSRApp(SummaryTab).use(getActivePinia()!),
      );
      expect(html).toContain(locale === "th" ? "แสดงเวลา" : "Show timestamps");
      expect(html).toContain(locale === "th" ? "ตั้งค่าการสรุป" : "Summary settings");
    },
  );

  it.each(["en", "th"])(
    "renders labelled controls and collapsible detail, with no request (%s)",
    async (locale) => {
      setLocale(locale);
      const store = useSummaryStore();
      const original = store.summary?.markdown;
      const html = await renderToString(
        createSSRApp(SummaryTab).use(getActivePinia()!),
      );
      expect(html).toContain("<details");
      expect(html).not.toContain("<details open");
      expect(html.indexOf("Ship it")).toBeLessThan(html.indexOf("Context"));
      expect(html).toContain(
        locale === "th" ? "ขยายพื้นที่อ่าน" : "Expand reading area",
      );
      expect(html).toContain(locale === "th" ? "คัดลอกบทสรุป" : "Copy summary");
      expect(html).toContain("test-model");
      expect(store.summary?.markdown).toBe(original);
      expect(store.isRunning).toBe(false);
    },
  );

  it("separates all citation buttons from the prose and keeps bold text and safe escaping", async () => {
    setLocale("en");
    const html = await renderToString(
      createSSRApp(SummaryLine, {
        showTimestamps: true,
        parts: parseInline(
          "Before [00:10] **owner** after [01:02:03] <script>bad()</script>",
        ),
      }),
    );
    expect(html).toContain('aria-label="Jump to 00:10"');
    expect(html).toContain('aria-label="Jump to 01:02:03"');
    expect(html.indexOf("after")).toBeLessThan(html.indexOf("<button"));
    expect(html).toContain("<strong");
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain("<script>");
  });
});
