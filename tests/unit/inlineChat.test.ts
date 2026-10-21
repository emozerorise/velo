import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, getActivePinia, setActivePinia } from "pinia";
import { nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event as TauriEvent } from "@tauri-apps/api/event";
import { chatService } from "@/services/chatService";
import { useChatStore } from "@/stores/chatStore";
import { usePlayerStore } from "@/stores/playerStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useSummaryStore } from "@/stores/summaryStore";
import { setLocale } from "@/composables/useI18n";
import InlineChat from "@/components/chat/InlineChat.vue";
import ChatComposer from "@/components/chat/ChatComposer.vue";
import ChatMessage from "@/components/chat/ChatMessage.vue";
import SummaryPromptEditor from "@/components/transcript/SummaryPromptEditor.vue";
import TranscriptPanel from "@/components/transcript/TranscriptPanel.vue";
import { useWorkspaceStore } from "@/stores/workspaceStore";
import { mountView, TestNode } from "./helpers/mountView";
import { mountComposable } from "./helpers/mountComposable";
import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@/services/chatService", () => ({
  chatService: {
    status: vi.fn(),
    send: vi.fn(),
    cancel: vi.fn().mockResolvedValue(undefined),
  },
  newRequestId: () => "synthetic-request",
}));
let chatEvents: Map<string, (event: TauriEvent<unknown>) => void>;
describe("inline chat handlers", () => {
  it("Stop exposes Retry and retries the original turn without losing newer draft edits", async () => {
    const chat = useChatStore(); await chat.prepare('/clip.wav');
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    chat.setDraft('Original question'); await nextTick(); view.find('chat-send').trigger('click'); await nextTick(); await nextTick();
    view.find('chat-stop').trigger('click'); await nextTick();
    chat.setDraft('New unsent edits'); await nextTick();
    view.find('chat-retry').trigger('click'); await nextTick(); await nextTick();
    expect(chat.messages.filter(message => message.role === 'user')).toHaveLength(1);
    expect(chatService.send).toHaveBeenLastCalledWith('/clip.wav', expect.any(String), 'Original question', []);
    expect(chat.draft).toBe('New unsent edits'); view.unmount();
  });
  beforeEach(async () => {
    setActivePinia(createPinia());
    setLocale("en");
    vi.clearAllMocks();
    vi.stubGlobal(
      "window",
      Object.assign(new EventTarget(), {
        innerWidth: 640,
        innerHeight: 400,
        devicePixelRatio: 2,
        matchMedia: () => new EventTarget(),
      }),
    );
    vi.stubGlobal("document", { activeElement: null });
    vi.stubGlobal("HTMLElement", TestNode);
    vi.stubGlobal("HTMLInputElement", class extends TestNode {});
    vi.stubGlobal("HTMLTextAreaElement", class extends TestNode {});
    vi.stubGlobal("requestAnimationFrame", () => 1);
    vi.stubGlobal("cancelAnimationFrame", () => {});
    vi.stubGlobal(
      "ResizeObserver",
      class {
        observe() {}
        disconnect() {}
      },
    );
    chatEvents = new Map();
    vi.mocked(listen).mockImplementation(async (name, handler) => {
      chatEvents.set(name, handler);
      return () => chatEvents.delete(name);
    });
    vi.mocked(chatService.status).mockResolvedValue({
      path: "/clip.wav",
      provider: "ollama",
      model: "synthetic",
      base_url: "http://localhost:11434",
      has_transcript: true,
      remote: false,
      configured: true,
      has_key: false,
    });
    vi.mocked(chatService.send).mockResolvedValue({
      path: "/clip.wav",
      request_id: "synthetic-request",
      sources: [],
      coverage: { transcript: "full", summary: "absent", omitted_pairs: 0 },
    });
    usePlayerStore().mediaInfo = {
      path: "/clip.wav",
      file_name: "clip.wav",
      duration: 60,
      audio_codec: "pcm",
      video_codec: null,
      width: 0,
      height: 0,
      fps: 0,
      hwdec_current: null,
    };
    await useChatStore().initListeners();
    await useChatStore().prepare("/clip.wav");
  });
  afterEach(() => {
    useChatStore().cleanupListeners();
    vi.unstubAllGlobals();
  });
  it.each(["en", "th"])(
    "starter_cannot_silently_replace_draft (%s)",
    async (locale) => {
      setLocale(locale);
      const chat = useChatStore();
      chat.setDraft("My current draft");
      const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
      view.find("starter-actions").trigger("click");
      await nextTick();
      expect(chat.draft).toBe("My current draft");
      expect(chatService.send).not.toHaveBeenCalled();
      view.find("keep-draft").trigger("click");
      await nextTick();
      expect(chat.draft).toBe("My current draft");
      view.find("starter-prompt").trigger("click");
      await nextTick();
      view.find("replace-draft").trigger("click");
      await nextTick();
      expect(chat.draft).not.toBe("My current draft");
      expect(chatService.send).not.toHaveBeenCalled();
      view.unmount();
    },
  );
  it("IME and Shift+Enter never Send; ordinary Enter sends once", async () => {
    const chat = useChatStore();
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    const field = view.find("chat-draft");
    field.value = "ถามเกี่ยวกับเพลง";
    field.trigger("input", { target: field });
    await nextTick();
    expect(chat.draft).toBe("ถามเกี่ยวกับเพลง");
    field.trigger("compositionstart");
    field.trigger(
      "keydown",
      Object.assign(new Event("keydown", { cancelable: true }), {
        key: "Enter",
        shiftKey: false,
        isComposing: true,
      }),
    );
    field.trigger("compositionend");
    field.trigger(
      "keydown",
      Object.assign(new Event("keydown", { cancelable: true }), {
        key: "Enter",
        shiftKey: true,
      }),
    );
    expect(chatService.send).not.toHaveBeenCalled();
    field.trigger(
      "keydown",
      Object.assign(new Event("keydown", { cancelable: true }), {
        key: "Enter",
        shiftKey: false,
        isComposing: false,
      }),
    );
    await nextTick();
    expect(chatService.send).toHaveBeenCalledOnce();
    view.unmount();
  });
  it("disabled composer cannot Send by click or keyboard", () => {
    const send = vi.fn();
    const view = mountView(
      ChatComposer,
      {
        modelValue: "Question",
        disabled: true,
        sending: false,
        remoteHost: null,
        onSend: send,
      },
      getActivePinia()!,
    );
    view.find("chat-send").trigger("click");
    view.find("chat-draft").trigger(
      "keydown",
      Object.assign(new Event("keydown", { cancelable: true }), {
        key: "Enter",
      }),
    );
    expect(send).not.toHaveBeenCalled();
    view.unmount();
  });
  it("save_prompt_does_not_generate and initial draft does not change saved instructions before submit", async () => {
    const settings = useSettingsStore();
    settings.settings.summary.instructions = "Saved old prompt";
    const view = mountView(
      SummaryPromptEditor,
      { open: true, initialDraft: "Suggested draft" },
      getActivePinia()!,
    );
    const field = view.nodes().find((n) => n.tagName === "TEXTAREA")!;
    expect(field.value).toBe("Suggested draft");
    expect(settings.settings.summary.instructions).toBe("Saved old prompt");
    const form = view.nodes().find((n) => n.tagName === "FORM")!;
    form.trigger("submit");
    await nextTick();
    await Promise.resolve();
    await vi.waitFor(() =>
      expect(settings.settings.summary.instructions).toBe("Suggested draft"),
    );
    expect(vi.mocked(invoke).mock.calls.map(([name]) => name)).toEqual([
      "settings_save",
    ]);
    view.unmount();
  });
  it("late_prompt_draft_does_not_overwrite_an_open_editor", async () => {
    const view = mountView(
      SummaryPromptEditor,
      { open: true, initialDraft: "First suggestion" },
      getActivePinia()!,
    );
    const field = view.nodes().find((node) => node.tagName === "TEXTAREA")!;
    field.value = "User edits";
    field.trigger("input");
    await nextTick();
    view.updateProps({ initialDraft: "Late suggestion" });
    await nextTick();
    expect(field.value).toBe("User edits");
    view.unmount();
  });
  it("prompt action proposes a draft without saving or generating", () => {
    const review = vi.fn();
    const view = mountView(
      ChatMessage,
      {
        message: {
          id: "prompt",
          role: "assistant",
          status: "complete",
          text: "```prompt\nOnly agreed actions\n```",
        },
        onPrompt: review,
      },
      getActivePinia()!,
    );
    view.find("use-chat-prompt").trigger("click");
    expect(review).toHaveBeenCalledWith("Only agreed actions");
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });
  it("a chat citation seeks to the absolute source time instead of adding an offset", () => {
    usePlayerStore().currentTime = 134;
    const view = mountView(
      ChatMessage,
      {
        message: {
          id: "citation",
          role: "assistant",
          status: "complete",
          text: "Check the source [00:37]",
          sources: [{ start: 37, end: 63 }],
        },
      },
      getActivePinia()!,
    );
    const citation = view
      .nodes()
      .find((node) => node.props["aria-label"] === "Jump to 00:37")!;
    citation.trigger("click");
    expect(invoke).toHaveBeenCalledWith("player_seek_absolute", {
      seconds: 37,
    });
    expect(invoke).not.toHaveBeenCalledWith("player_seek", expect.anything());
    view.unmount();
  });
  it("stream updates do not move a reader away from older content", async () => {
    const scroll = Object.assign(new TestNode("DIV"), {
      scrollHeight: 1200,
      clientHeight: 300,
      scrollTop: 0,
    });
    const view = mountView(
      InlineChat,
      { visible: true, scrollElement: scroll },
      getActivePinia()!,
    );
    await nextTick();
    scroll.scrollTop = 100;
    scroll.dispatchEvent(new Event("scroll"));
    const field = view.find("chat-draft");
    field.value = "Question";
    field.trigger("input", { target: field });
    await nextTick();
    view.find("chat-send").trigger("click");
    await nextTick();
    await nextTick();
    expect(scroll.scrollTop).toBe(100);
    expect(view.find("chat-latest")).toBeDefined();
    view.find("chat-latest").trigger("click");
    expect(scroll.scrollTop).toBe(1200);
    view.unmount();
  });
  it("cached transcript without summary or speech model supports a non-meeting question", async () => {
    useSummaryStore().summary = null;
    useTranscriptStore().engine = {
      ready: false,
      model_path: null,
      model_name: "missing",
      model_bytes: 0,
      downloading: false,
      model_managed: false,
    };
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    const field = view.find("chat-draft");
    field.value = "เพลงนี้พูดถึงประเด็นอะไรบ้าง";
    field.trigger("input", { target: field });
    await nextTick();
    expect(view.find("chat-send").props.disabled).toBe(false);
    view.find("chat-send").trigger("click");
    await nextTick();
    expect(chatService.send).toHaveBeenCalledWith(
      "/clip.wav",
      "synthetic-request",
      "เพลงนี้พูดถึงประเด็นอะไรบ้าง",
      [],
    );
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });
  it.each(["en", "th"])(
    "critical privacy and input copy use readable contrast tokens (%s)",
    (locale) => {
      setLocale(locale);
      const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
      const paragraphs = view.nodes().filter((node) => node.tagName === "P");
      const privacy = paragraphs.find((node) =>
        node.text.includes(
          locale === "th" ? "คำขอข้อความอยู่ในเครื่อง" : "Text requests stay",
        ),
      )!;
      const retention = paragraphs.find((node) =>
        node.text.includes(
          locale === "th"
            ? "บทสนทนาเก็บในหน่วยความจำ"
            : "This conversation is kept",
        ),
      )!;
      expect(privacy.props.class).toContain("text-fg/65");
      expect(retention.props.class).toContain("text-fg/60");
      expect(view.find("chat-draft").props.class).toContain(
        "placeholder:text-fg/60",
      );
      view.unmount();
    },
  );
  it("640x400 IME editing and compact tools retain a hidden streaming conversation", async () => {
    useTranscriptStore().transcript = {
      path: "/clip.wav",
      model: "cached",
      language: "th",
      prompt: "",
      created_at: 0,
      segments: [{ start: 0, end: 60, text: "เนื้อหาสมมติจากเพลง" }],
    };
    const chat = useChatStore();
    const workspace = useWorkspaceStore();
    const view = mountView(TranscriptPanel, {}, getActivePinia()!);
    view.find("tool-chat").trigger("click");
    await nextTick();
    const field = view.find("chat-draft");
    field.value = "เพลงนี้มีประเด็นอะไร";
    field.trigger("input", { target: field });
    await nextTick();
    field.trigger("compositionstart");
    field.trigger(
      "keydown",
      Object.assign(new Event("keydown", { cancelable: true }), {
        key: "Enter",
        isComposing: true,
      }),
    );
    expect(chatService.send).not.toHaveBeenCalled();
    field.trigger("compositionend");
    view.find("chat-send").trigger("click");
    await nextTick();
    field.value = "ร่างคำถามถัดไป";
    field.trigger("input", { target: field });
    await nextTick();
    view.find("tool-player").trigger("click");
    await nextTick();
    expect(workspace.compact).toBe(true);
    expect(workspace.chatVisible).toBe(false);
    const section = view
      .nodes()
      .find((node) => node.props.id === "workspace-chat")!;
    expect(section.style.display).toBe("none");
    chatEvents.get("velo://chat-delta")!({
      event: "velo://chat-delta",
      id: 0,
      payload: {
        path: "/clip.wav",
        request_id: "synthetic-request",
        sequence: 1,
        text: "คำตอบยังสตรีมอยู่",
      },
    });
    view.find("tool-chat").trigger("click");
    await nextTick();
    expect(section.style.display).not.toBe("none");
    expect(view.find("chat-draft")).toBe(field);
    expect(chat.draft).toBe("ร่างคำถามถัดไป");
    expect(chat.messages.at(-1)?.text).toBe("คำตอบยังสตรีมอยู่");
    expect(chat.isRunning).toBe(true);
    expect(chatService.cancel).not.toHaveBeenCalled();
    view.unmount();
  });
  it("composer exposes a visible focus ring and a solid contrast Send control", () => {
    const view = mountView(
      ChatComposer,
      {
        modelValue: "Question",
        disabled: false,
        sending: false,
        remoteHost: null,
      },
      getActivePinia()!,
    );
    expect(view.find("chat-draft").parent?.props.class).toContain(
      "focus-within:ring-2",
    );
    expect(view.find("chat-send").props.class).toContain("bg-blue-600");
    view.unmount();
  });
  it("prompt dialog is named and owns header-button Space and Tab focus", async () => {
    const dispose = mountComposable(() =>
      useKeyboardShortcuts({ register: true }),
    );
    const view = mountView(
      SummaryPromptEditor,
      { open: true, initialDraft: "Review me" },
      getActivePinia()!,
    );
    const dialog = view.nodes().find((node) => node.props.role === "dialog");
    expect(dialog?.props["aria-modal"]).toBe("true");
    expect(dialog?.props["aria-label"]).toBe("Your summary prompt");
    const close = view
      .nodes()
      .find(
        (node) =>
          node.tagName === "BUTTON" && node.props["aria-label"] === "Close",
      )!;
    const event = Object.assign(new Event("keydown", { cancelable: true }), {
      key: " ",
      code: "Space",
    });
    close.trigger("keydown", event);
    if (!event.cancelBubble) window.dispatchEvent(event);
    expect(invoke).not.toHaveBeenCalled();
    const save = view
      .nodes()
      .find(
        (node) => node.tagName === "BUTTON" && node.props.type === "submit",
      )!;
    save.focus();
    const tab = Object.assign(new Event("keydown", { cancelable: true }), {
      key: "Tab",
      shiftKey: false,
    });
    save.trigger("keydown", tab);
    expect(tab.defaultPrevented).toBe(true);
    expect(close.focusCount).toBeGreaterThan(0);
    view.unmount();
    dispose();
    await nextTick();
  });
});
