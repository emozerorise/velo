<template>
  <article
    class="flex min-w-0"
    :class="user ? 'justify-end' : 'justify-start'"
    :aria-label="t(user ? 'chat.you' : 'chat.assistant')"
  >
    <div
      v-if="user"
      class="flex min-w-0 max-w-[90%] sm:max-w-[78%] flex-col items-end gap-2"
    >
      <p class="flex items-center gap-1.5 pr-1 text-xs font-medium text-fg/60">
        <UserRound class="w-3.5 h-3.5" aria-hidden="true" />{{ t("chat.you") }}
      </p>
      <div
        class="chat-user-bubble min-w-0 rounded-[22px] rounded-tr-md border border-accent/15 bg-accent/[0.08] px-4 py-3 sm:px-5 sm:py-3.5 shadow-[0_2px_8px_rgb(0_0_0/0.025)]"
      >
        <p
          class="reading-text whitespace-pre-wrap break-words text-fg/90 [overflow-wrap:anywhere]"
        >
          {{ message.text }}
        </p>
      </div>
    </div>
    <div v-else class="w-full min-w-0 space-y-3">
      <header class="flex flex-wrap items-center gap-x-2.5 gap-y-2">
        <span
          class="grid h-8 w-8 shrink-0 place-items-center rounded-xl border border-accent/20 bg-accent/[0.08] text-accent shadow-[0_2px_6px_rgb(0_0_0/0.03)]"
          ><AudioLines class="w-[18px] h-[18px]" aria-hidden="true"
        /></span>
        <p class="text-sm font-semibold text-fg/85">
          {{ t("chat.assistant") }}
        </p>
        <span
          v-if="message.status === 'streaming'"
          role="status"
          class="flex items-center gap-1.5 rounded-full bg-accent/[0.08] px-2.5 py-1 text-xs text-accent"
        >
          <LoaderCircle
            class="w-3 h-3 animate-spin motion-reduce:animate-none"
            aria-hidden="true"
          />{{ t("chat.answering") }}
        </span>
      </header>
      <div class="sm:ml-[42px] min-w-0 space-y-3">
        <div
          class="chat-assistant-bubble min-w-0 rounded-[22px] rounded-tl-md border border-fg/10 bg-chrome px-4 py-4 sm:px-6 sm:py-5 shadow-[0_3px_16px_rgb(0_0_0/0.035)]"
        >
          <div
            class="chat-answer reading-text space-y-4 text-fg/85 [overflow-wrap:anywhere]"
          >
            <template v-for="(node, index) in nodes" :key="index">
              <pre
                v-if="node.type === 'code'"
                class="rounded-xl border border-fg/10 bg-fg/[0.035] p-4 overflow-x-auto whitespace-pre-wrap break-words text-sm leading-relaxed"
              ><code>{{ node.text }}</code></pre>
              <component
                :is="`h${Math.min(6, Math.max(3, node.level + 1))}`"
                v-else-if="node.type === 'heading'"
                class="chat-answer-heading font-semibold tracking-tight text-fg/95"
                :class="
                  node.level <= 2 ? 'text-lg leading-7' : 'text-base leading-7'
                "
              >
                <ChatInline :parts="node.parts" @seek="seek" />
              </component>
              <ul
                v-else-if="node.type === 'list'"
                class="list-disc pl-5 space-y-2 marker:text-accent/70"
              >
                <li v-for="(parts, item) in node.items" :key="item">
                  <ChatInline :parts="parts" @seek="seek" />
                </li>
              </ul>
              <p v-else class="whitespace-pre-wrap">
                <ChatInline :parts="node.parts" @seek="seek" />
              </p>
            </template>
            <span
              v-if="message.status === 'streaming' && !message.text"
              class="flex gap-1.5 py-2"
              aria-hidden="true"
            >
              <span
                v-for="dot in 3"
                :key="dot"
                class="w-1.5 h-1.5 rounded-full bg-accent/50"
              />
            </span>
          </div>
        </div>
        <div
          v-if="
            message.coverage ||
            draft ||
            message.status === 'failed' ||
            message.status === 'cancelled'
          "
          class="flex flex-wrap items-center gap-x-3 gap-y-2 px-1 text-xs leading-relaxed text-fg/60"
        >
          <p
            v-if="message.status === 'failed' || message.status === 'cancelled'"
            class="basis-full"
          >
            {{
              t(
                message.status === "failed"
                  ? "chat.partialFailed"
                  : "chat.partialCancelled",
              )
            }}
          </p>
          <p v-if="message.coverage" class="flex-1 min-w-0">
            {{
              t(
                message.coverage.transcript === "full"
                  ? "chat.full"
                  : "chat.excerpts",
              )
            }}
            · {{ t(`chat.summary.${message.coverage.summary}`)
            }}<template v-if="omitted">
              · {{ t("chat.omitted", { count: omitted }) }}</template
            >
          </p>
          <button
            v-if="draft"
            type="button"
            data-testid="use-chat-prompt"
            class="text-xs text-accent rounded-lg border border-accent/20 px-3 py-2 hover:bg-accent/10 focus-visible:outline focus-visible:outline-accent"
            @click="emit('prompt', draft)"
          >
            {{ t("chat.usePrompt") }}
          </button>
        </div>
      </div>
    </div>
  </article>
</template>
<script setup lang="ts">
import { computed } from "vue";
import { AudioLines, LoaderCircle, UserRound } from "@lucide/vue";
import type { ChatMessage as Message } from "@/types/chat";
import { parseChatText, promptDraft } from "@/utils/chatContent";
import { usePlayerStore } from "@/stores/playerStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useI18n } from "@/composables/useI18n";
import ChatInline from "./ChatInline.vue";
const props = defineProps<{ message: Message }>();
const emit = defineEmits<{ (event: "prompt", text: string): void }>();
const { t } = useI18n();
const player = usePlayerStore();
const settings = useSettingsStore();
const user = computed(() => props.message.role === "user");
const nodes = computed(() =>
  parseChatText(
    props.message.text,
    props.message.sources ?? [],
    settings.settings.general.show_summary_timestamps,
  ),
);
const draft = computed(() =>
  props.message.status === "complete" ? promptDraft(props.message.text) : null,
);
const omitted = computed(
  () =>
    (props.message.coverage?.omitted_pairs ?? 0) +
    (props.message.clientOmittedPairs ?? 0),
);
function seek(seconds: number) {
  player.seekAbsolute(seconds);
}
</script>
<style scoped>
.chat-answer-heading:not(:first-child) {
  padding-top: 0.5rem;
}
</style>
