import { describe, expect, it } from "vitest";
import { parseChatText, promptDraft } from "@/utils/chatContent";
import { createSSRApp } from "vue";
import { createPinia } from "pinia";
import { renderToString } from "vue/server-renderer";
import ChatMessage from "@/components/chat/ChatMessage.vue";
describe("safe generated conversation text", () => {
  it("parses assistant headings and emphasis without hiding incomplete streamed markers", () => {
    expect(
      parseChatText(
        "## สรุปเนื้อหา\n\n**ภาพรวม:** นัดตรวจงาน\n\n- **งานถัดไป** ยืนยันข้อมูล",
        [],
        true,
      ),
    ).toEqual([
      {
        type: "heading",
        level: 2,
        parts: [{ type: "text", text: "สรุปเนื้อหา" }],
      },
      {
        type: "paragraph",
        parts: [
          { type: "strong", parts: [{ type: "text", text: "ภาพรวม:" }] },
          { type: "text", text: " นัดตรวจงาน" },
        ],
      },
      {
        type: "list",
        items: [
          [
            { type: "strong", parts: [{ type: "text", text: "งานถัดไป" }] },
            { type: "text", text: " ยืนยันข้อมูล" },
          ],
        ],
      },
    ]);
    expect(parseChatText("กำลังเขียน **ข้อมูลยังไม่ครบ", [], true)).toEqual([
      {
        type: "paragraph",
        parts: [{ type: "text", text: "กำลังเขียน **ข้อมูลยังไม่ครบ" }],
      },
    ]);
  });
  it("keeps fenced prompt contents literal while formatting surrounding headings", () => {
    expect(
      parseChatText(
        "### แนวทาง\n```prompt\n## Literal\n**Keep this** [00:05]\n```",
        [{ start: 0, end: 10 }],
        true,
      ),
    ).toEqual([
      { type: "heading", level: 3, parts: [{ type: "text", text: "แนวทาง" }] },
      {
        type: "code",
        language: "prompt",
        text: "## Literal\n**Keep this** [00:05]",
      },
    ]);
  });
  it("renders structured assistant typography, escapes emphasized HTML, and keeps user input literal", async () => {
    const render = (role: "assistant" | "user", text: string) =>
      renderToString(
        createSSRApp(ChatMessage, {
          message: { id: "formatted", role, text, status: "complete" },
        }).use(createPinia()),
      );
    const html = await render(
      "assistant",
      "## สรุปเนื้อหา\n\n**ภาพรวม:** ตรวจงาน\n\n### ขั้นต่อไป\n\n**<img src=x onerror=alert(1)>** [outside](https://example.com)",
    );
    expect(html).toMatch(/<h3\b[^>]*>[\s\S]*สรุปเนื้อหา[\s\S]*<\/h3>/);
    expect(html).toMatch(/<h4\b[^>]*>[\s\S]*ขั้นต่อไป[\s\S]*<\/h4>/);
    expect(html).toMatch(/<strong\b[^>]*>[\s\S]*ภาพรวม:[\s\S]*<\/strong>/);
    expect(html).not.toContain("**");
    expect(html).not.toContain("##");
    expect(html).toContain("&lt;img");
    expect(html).not.toContain("<img");
    expect(html).not.toContain("<a ");
    const user = await render("user", "## คำถาม **พิมพ์เอง**");
    expect(user).toContain("## คำถาม **พิมพ์เอง**");
    expect(user).not.toContain("<strong");
    expect(user).not.toContain("<h3");
  });
  it("announces actual streaming with partial text and removes the active status on completion", async () => {
    const render = (status: "streaming" | "complete") =>
      renderToString(
        createSSRApp(ChatMessage, {
          message: {
            id: "stream",
            role: "assistant",
            text: "คำตอบบางส่วน",
            status,
          },
        }).use(createPinia()),
      );
    expect(await render("streaming")).toContain('role="status"');
    expect(await render("complete")).not.toContain('role="status"');
  });
  it("resolves supplied fractional start labels to the exact source start without accepting unrelated stamps", () => {
    for (const source of [
      { start: 10.42, end: 12.7 },
      { start: 10.92, end: 10.99 },
    ]) {
      const nodes = parseChatText("[00:10] [00:09]", [source], true);
      const parts = nodes.flatMap((node) =>
        node.type === "paragraph" ? node.parts : [],
      );
      expect(parts.filter((part) => part.type === "citation")).toEqual([
        { type: "citation", label: "00:10", seconds: source.start },
      ]);
      expect(JSON.stringify(parts)).toContain("[00:09]");
      expect(
        JSON.stringify(parseChatText("[00:10]", [source], false)),
      ).not.toContain("00:10");
    }
  });
  it("renders model HTML and external links as escaped readable text", async () => {
    const html = await renderToString(
      createSSRApp(ChatMessage, {
        message: {
          id: "safe",
          role: "assistant",
          status: "complete",
          text: "<script>alert(1)</script> [outside](https://example.com)",
        },
      }).use(createPinia()),
    );
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain("<script>");
    expect(html).not.toContain("<a ");
  });
  it("html_and_external_links_are_text", () => {
    const nodes = parseChatText(
      "<script>alert(1)</script>\n\n[link](https://example.com)\n- item",
      [],
      true,
    );
    expect(JSON.stringify(nodes)).toContain("<script>");
    expect(JSON.stringify(nodes)).toContain("https://example.com");
    expect(nodes.map((node) => node.type)).toEqual([
      "paragraph",
      "paragraph",
      "list",
    ]);
    expect(JSON.stringify(nodes)).not.toContain('"href"');
  });
  it("unsupported_citation_not_seekable", () => {
    const text = "Supported [00:05], unsupported [10:00], malformed [00:65]";
    const nodes = parseChatText(text, [{ start: 0, end: 30 }], true);
    const parts = nodes.flatMap((node) =>
      node.type === "paragraph" ? node.parts : [],
    );
    expect(parts.filter((part) => part.type === "citation")).toEqual([
      { type: "citation", label: "00:05", seconds: 5 },
    ]);
    expect(JSON.stringify(parts)).toContain("[10:00]");
    expect(JSON.stringify(parts)).toContain("[00:65]");
    expect(
      JSON.stringify(parseChatText(text, [{ start: 0, end: 30 }], false)),
    ).not.toContain('"citation"');
  });
  it("code never becomes a seek control and incomplete fences remain readable", () => {
    const nodes = parseChatText(
      "```prompt\nUse [00:05] <script>\n```",
      [{ start: 0, end: 30 }],
      true,
    );
    expect(nodes).toEqual([
      { type: "code", language: "prompt", text: "Use [00:05] <script>" },
    ]);
    expect(parseChatText("```txt\npartial", [], true)).toEqual([
      { type: "code", language: "txt", text: "partial" },
    ]);
  });
  it("two_prompt_blocks_not_auto_applied", () => {
    expect(promptDraft("```prompt\nOne\n```\n```prompt\nTwo\n```")).toBeNull();
    expect(promptDraft("```prompt\nUnfinished")).toBeNull();
    expect(promptDraft("```prompt\n" + "ก".repeat(3000) + "\n```")).toBeNull();
    expect(
      promptDraft("Try this:\n```prompt\nKeep only agreed actions\n```"),
    ).toBe("Keep only agreed actions");
  });
});
