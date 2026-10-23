import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, getActivePinia, setActivePinia } from "pinia";
import { nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import SummaryPromptEditor from "@/components/transcript/SummaryPromptEditor.vue";
import { useSettingsStore } from "@/stores/settingsStore";
import { setLocale } from "@/composables/useI18n";
import { mountView, TestNode } from "./helpers/mountView";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe("summary format editor controls", () => {
  it.each([false, true])(
    "focuses format selection on normal open, but the draft when reviewing a chat proposal (%s)",
    async (proposal) => {
      const view = mountView(
        SummaryPromptEditor,
        {
          open: true,
          ...(proposal ? { initialDraft: "Proposed instructions" } : {}),
        },
        getActivePinia()!,
      );
      await nextTick();
      expect((document.activeElement as unknown as TestNode).tagName).toBe(
        proposal ? "TEXTAREA" : "INPUT",
      );
      view.unmount();
    },
  );
  beforeEach(() => {
    setActivePinia(createPinia());
    setLocale("en");
    vi.clearAllMocks();
    vi.stubGlobal("window", new EventTarget());
    vi.stubGlobal("document", { activeElement: null });
    vi.stubGlobal("HTMLElement", TestNode);
    vi.stubGlobal("Document", class {});
    vi.stubGlobal("ShadowRoot", class {});
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });
  const field = (view: ReturnType<typeof mountView>, tag: string) =>
    view.nodes().find((node) => node.tagName === tag)!;
  const choose = (view: ReturnType<typeof mountView>, value: string) =>
    view
      .nodes()
      .find(
        (n) =>
          n.tagName === "INPUT" &&
          n.props.name === "summary-format" &&
          n.props.value === value,
      )!
      .trigger("change");
  const selected = (view: ReturnType<typeof mountView>) =>
    view
      .nodes()
      .find(
        (n) =>
          n.tagName === "INPUT" &&
          n.props.name === "summary-format" &&
          n.props.checked,
      )!;

  it("keeps keyboard focus inside visible controls and restores the opener after Escape", async () => {
    const opener = new TestNode("BUTTON");
    const root = new TestNode("ROOT");
    root.children.push(opener);
    opener.parent = root;
    opener.focus();
    const close = vi.fn();
    const view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close },
      getActivePinia()!,
    );
    await nextTick();
    const header = view
      .nodes()
      .find(
        (n) => n.tagName === "BUTTON" && n.props["aria-label"] === "Close",
      )!;
    const submit = view.nodes().find((n) => n.props.type === "submit")!;
    expect(
      view.nodes().find((n) => n.tagName === "SUMMARY")?.props.tabindex,
    ).toBe("0");
    const key = (node: TestNode, value: string, shiftKey = false) =>
      node.trigger(
        "keydown",
        Object.assign(new Event("keydown", { cancelable: true }), {
          key: value,
          shiftKey,
        }),
      );
    header.focus();
    key(header, "Tab", true);
    expect(document.activeElement).toBe(submit);
    key(submit, "Tab");
    expect(document.activeElement).toBe(header);
    const advanced = view.find("summary-advanced-toggle");
    advanced.trigger("click");
    await nextTick();
    expect(view.find("summary-advanced-fields").getClientRects()).toHaveLength(
      1,
    );
    const textarea = field(view, "TEXTAREA");
    textarea.focus();
    key(textarea, "Escape");
    expect(close).toHaveBeenCalledOnce();
    view.unmount();
    expect(document.activeElement).toBe(opener);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("selects detail independently without replacing text or saving until explicitly submitted", async () => {
    const settings = useSettingsStore();
    settings.settings.summary.instructions = "Keep these personal instructions";
    const view = mountView(
      SummaryPromptEditor,
      { open: true },
      getActivePinia()!,
    );
    await nextTick();
    expect(view.find("summary-advanced-fields").style.display).toBe("none");
    expect(
      view
        .nodes()
        .filter(
          (n) => n.tagName === "INPUT" && n.props.name === "summary-format",
        ),
    ).toHaveLength(3);
    view.find("summary-detail-detailed").trigger("change");
    await nextTick();
    expect(settings.settings.summary.meeting_detail).toBe("standard");
    expect(field(view, "TEXTAREA").value).toBe(
      "Keep these personal instructions",
    );
    choose(view, "custom");
    await nextTick();
    expect(view.find("summary-advanced-fields").style.display).not.toBe("none");
    choose(view, "meeting");
    await nextTick();
    expect(view.find("summary-detail-detailed").props.checked).toBe(true);
    expect(invoke).not.toHaveBeenCalled();
    field(view, "FORM").trigger("submit");
    await vi.waitFor(() =>
      expect(settings.settings.summary.meeting_detail).toBe("detailed"),
    );
    expect(vi.mocked(invoke).mock.calls.map(([name]) => name)).toEqual([
      "settings_save",
    ]);
    view.unmount();
  });

  it("asks before an example replaces nonempty text and supports keeping the current draft", async () => {
    const view = mountView(
      SummaryPromptEditor,
      { open: true, initialDraft: "Do not lose this draft" },
      getActivePinia()!,
    );
    choose(view, "custom");
    await nextTick();
    const example = () =>
      view
        .nodes()
        .find((n) => n.tagName === "BUTTON" && n.text === "One paragraph")!;
    example().trigger("click");
    await nextTick();
    expect(field(view, "TEXTAREA").value).toBe("Do not lose this draft");
    view.find("summary-example-keep").trigger("click");
    await nextTick();
    expect(field(view, "TEXTAREA").value).toBe("Do not lose this draft");
    example().trigger("click");
    await nextTick();
    view.find("summary-example-replace").trigger("click");
    await nextTick();
    expect(field(view, "TEXTAREA").value).toContain("without headings");
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });

  it("keeps draft edits across format choices; Cancel and reopening restore only saved options", async () => {
    const settings = useSettingsStore();
    settings.settings.summary.instructions = "Saved instructions";
    const close = vi.fn();
    let view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close },
      getActivePinia()!,
    );
    choose(view, "custom");
    const textarea = field(view, "TEXTAREA");
    textarea.value = "My unsaved structure";
    textarea.trigger("input");
    await nextTick();
    choose(view, "key_points");
    await nextTick();
    expect(textarea.value).toBe("My unsaved structure");
    expect(settings.settings.summary.format).toBe("meeting");
    expect(settings.settings.summary.instructions).toBe("Saved instructions");
    view
      .nodes()
      .find((node) => node.tagName === "BUTTON" && node.text === "Cancel")!
      .trigger("click");
    expect(close).toHaveBeenCalledOnce();
    view.unmount();
    view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close },
      getActivePinia()!,
    );
    await nextTick();
    expect(selected(view).props.value).toBe("meeting");
    expect(field(view, "TEXTAREA").value).toBe("Saved instructions");
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });

  it("commits format and instructions only after disk success, without starting inference", async () => {
    let finish!: () => void;
    vi.mocked(invoke).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = () => resolve(undefined);
        }),
    );
    const settings = useSettingsStore();
    const close = vi.fn();
    const view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close, initialDraft: "One paragraph" },
      getActivePinia()!,
    );
    choose(view, "custom");
    field(view, "FORM").trigger("submit");
    await nextTick();
    expect(settings.settings.summary.format).toBe("meeting");
    expect(settings.settings.summary.instructions).toBe("");
    expect(selected(view).props.disabled).toBe(true);
    expect(close).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("settings_save", {
      settings: expect.objectContaining({
        summary: expect.objectContaining({
          format: "custom",
          instructions: "One paragraph",
        }),
      }),
    });
    finish();
    await vi.waitFor(() => expect(close).toHaveBeenCalledOnce());
    expect(settings.settings.summary.format).toBe("custom");
    expect(settings.settings.summary.instructions).toBe("One paragraph");
    expect(vi.mocked(invoke).mock.calls.map(([name]) => name)).toEqual([
      "settings_save",
    ]);
    view.unmount();
  });

  it("retains the saved format and prompt after a failed save, leaving edits available for retry", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValueOnce(new Error("Disk full"));
    const settings = useSettingsStore();
    const close = vi.fn();
    const view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close, initialDraft: "New instructions" },
      getActivePinia()!,
    );
    choose(view, "custom");
    field(view, "FORM").trigger("submit");
    await vi.waitFor(() =>
      expect(view.nodes().some((node) => node.props.role === "alert")).toBe(
        true,
      ),
    );
    expect(settings.settings.summary.format).toBe("meeting");
    expect(settings.settings.summary.instructions).toBe("");
    expect(selected(view).props.value).toBe("custom");
    expect(field(view, "TEXTAREA").value).toBe("New instructions");
    expect(close).not.toHaveBeenCalled();
    view.unmount();
  });

  it("offers key-point examples that do not contradict its format, and free-form examples for custom", async () => {
    const view = mountView(
      SummaryPromptEditor,
      { open: true },
      getActivePinia()!,
    );
    choose(view, "key_points");
    await nextTick();
    expect(
      view
        .nodes()
        .some(
          (node) =>
            node.tagName === "BUTTON" && node.text === "Concise key points",
        ),
    ).toBe(true);
    expect(
      view
        .nodes()
        .some(
          (node) => node.tagName === "BUTTON" && node.text === "One paragraph",
        ),
    ).toBe(false);
    choose(view, "custom");
    await nextTick();
    view
      .nodes()
      .find(
        (node) => node.tagName === "BUTTON" && node.text === "One paragraph",
      )!
      .trigger("click");
    await nextTick();
    expect(field(view, "TEXTAREA").value).toContain("without headings");
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });
});
