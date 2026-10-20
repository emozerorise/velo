<template>
  <article
    class="space-y-3"
    :class="
      message.role === 'user'
        ? 'ml-auto max-w-[90%] rounded-2xl bg-fg/[0.055] px-5 py-4'
        : 'py-4'
    "
    :aria-label="t(message.role === 'user' ? 'chat.you' : 'chat.assistant')"
  >
    <p class="text-xs font-medium text-fg/60">
      {{ t(message.role === 'user' ? 'chat.you' : 'chat.assistant') }}
    </p>
    <template v-for="(node, index) in nodes" :key="index">
      <pre
        v-if="node.type === 'code'"
        class="rounded-xl border border-fg/10 bg-inset/30 p-4 overflow-x-auto whitespace-pre-wrap break-words text-sm leading-relaxed"
      ><code>{{ node.text }}</code></pre>
      <ul
        v-else-if="node.type === 'list'"
        class="list-disc pl-5 space-y-2 text-base leading-relaxed text-fg/80"
      >
        <li v-for="(parts, item) in node.items" :key="item">
          <template v-for="(part, n) in parts" :key="n"
            ><button
              v-if="part.type === 'citation'"
              class="chat-citation"
              :aria-label="t('summary.jump', { time: part.label })"
              @click="seek(part.seconds)"
            >
              {{ part.label }}</button
            ><span v-else class="whitespace-pre-wrap">{{
              part.text
            }}</span></template
          >
        </li>
      </ul>
      <p
        v-else
        class="text-base leading-relaxed text-fg/80 break-words whitespace-pre-wrap"
      >
        <template v-for="(part, n) in node.parts" :key="n"
          ><button
            v-if="part.type === 'citation'"
            class="chat-citation"
            :aria-label="t('summary.jump', { time: part.label })"
            @click="seek(part.seconds)"
          >
            {{ part.label }}</button
          ><span v-else>{{ part.text }}</span></template
        >
      </p>
    </template>
    <p
      v-if="message.status === 'streaming' && !message.text"
      role="status"
      class="text-sm text-fg/60"
    >
      {{ t('chat.answering') }}
    </p>
    <p
      v-if="message.status === 'failed' || message.status === 'cancelled'"
      class="text-xs text-fg/60"
    >
      {{
        t(
          message.status === 'failed'
            ? 'chat.partialFailed'
            : 'chat.partialCancelled',
        )
      }}
    </p>
    <p v-if="message.coverage" class="text-xs leading-relaxed text-fg/60">
      {{
        t(
          message.coverage.transcript === 'full'
            ? 'chat.full'
            : 'chat.excerpts',
        )
      }}
      · {{ t(`chat.summary.${message.coverage.summary}`)
      }}<template v-if="omitted">
        · {{ t('chat.omitted', { count: omitted }) }}</template
      >
    </p>
    <button
      v-if="draft"
      data-testid="use-chat-prompt"
      class="text-sm text-accent rounded-lg border border-accent/20 px-3 py-2 hover:bg-accent/10 focus-visible:outline focus-visible:outline-accent"
      @click="emit('prompt', draft)"
    >
      {{ t('chat.usePrompt') }}
    </button>
  </article>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import type { ChatMessage as Message } from '@/types/chat';
import { parseChatText, promptDraft, type ChatNode } from '@/utils/chatContent';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
const props = defineProps<{ message: Message }>();
const emit = defineEmits<{ (event: 'prompt', text: string): void }>();
const { t } = useI18n();
const player = usePlayerStore();
const settings = useSettingsStore();
const nodes = computed<ChatNode[]>(() =>
  props.message.role === 'user'
    ? [
        {
          type: 'paragraph',
          parts: [{ type: 'text', text: props.message.text }],
        },
      ]
    : parseChatText(
        props.message.text,
        props.message.sources ?? [],
        settings.settings.general.show_summary_timestamps,
      ),
);
const draft = computed(() =>
  props.message.role === 'assistant' && props.message.status === 'complete'
    ? promptDraft(props.message.text)
    : null,
);
const omitted = computed(
  () =>
    (props.message.coverage?.omitted_pairs ?? 0) +
    (props.message.clientOmittedPairs ?? 0),
);
function seek(seconds: number) {
  void player.seek(seconds, false);
}
</script>
<style scoped>
.chat-citation {
  display: inline-block;
  margin: 0 3px;
  padding: 1px 6px;
  border-radius: 6px;
  color: rgb(var(--velo-accent));
  background: rgb(var(--velo-accent) / 0.08);
  font:
    12px ui-monospace,
    monospace;
}
.chat-citation:focus-visible {
  outline: 2px solid rgb(var(--velo-accent));
}
</style>
