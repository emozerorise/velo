<template>
  <section
    id="workspace-chat"
    class="w-full max-w-[58rem] mx-auto px-5 sm:px-8 py-8 space-y-5"
    :aria-label="t('workspace.chatTitle')"
  >
    <header
      ref="heading"
      class="flex items-center gap-3 border-t border-fg/10 pt-7"
    >
      <AudioLines class="w-5 h-5 text-accent" aria-hidden="true" />
      <h2 class="text-lg font-semibold text-fg/90">
        {{ t('workspace.chatTitle') }}
      </h2>
      <button
        data-testid="close-chat"
        class="ml-auto p-2 rounded-lg text-fg/60 hover:text-fg focus-visible:outline focus-visible:outline-accent"
        :aria-label="t('chat.close')"
        @click="emit('close')"
      >
        <X class="w-4 h-4" aria-hidden="true" />
      </button>
    </header>
    <template v-if="!chat.messages.length">
      <h3
        class="text-2xl sm:text-[28px] font-semibold tracking-tight text-fg/90"
      >
        {{ t('chat.explore') }}
      </h3>
      <p class="text-base leading-relaxed text-fg/60">
        {{ t('chat.exploreHint') }}
      </p>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="starter in starters"
          :key="starter.id"
          :data-testid="`starter-${starter.id}`"
          class="px-4 py-3 rounded-xl border border-fg/10 text-sm text-fg/70 hover:bg-accent/[0.06] hover:border-accent/25 focus-visible:outline focus-visible:outline-accent"
          @click="choose(t(starter.prompt))"
        >
          {{ t(starter.label) }}
        </button>
      </div>
    </template>
    <div
      v-if="replacement !== null"
      role="group"
      :aria-label="t('chat.replaceQuestion')"
      class="flex flex-wrap items-center gap-3 rounded-xl border border-accent/20 p-3 text-sm"
    >
      <p class="flex-1 text-fg/70">{{ t('chat.replaceQuestion') }}</p>
      <button
        data-testid="keep-draft"
        class="text-fg/65 p-2 rounded-lg hover:bg-fg/5"
        @click="replacement = null"
      >
        {{ t('chat.keepDraft') }}</button
      ><button
        data-testid="replace-draft"
        class="text-accent p-2 rounded-lg hover:bg-accent/10"
        @click="replace"
      >
        {{ t('chat.replaceDraft') }}
      </button>
    </div>
    <div class="space-y-8 sm:space-y-10 select-text">
      <ChatMessage
        v-for="message in chat.messages"
        :key="message.id"
        :message="message"
        @prompt="reviewPrompt"
      />
    </div>
    <button
      v-if="unread"
      data-testid="chat-latest"
      class="text-sm text-accent border border-accent/20 rounded-full px-4 py-2"
      @click="latest"
    >
      {{ t('chat.latest') }}
    </button>
    <p v-if="chat.preparing" role="status" class="text-sm text-fg/60">
      {{ t('chat.preparing') }}
    </p>
    <template v-else>
      <p
        v-if="!chat.status?.has_transcript"
        class="text-sm leading-relaxed text-fg/60"
      >
        {{ t('chat.needsTranscript') }}
        <button class="text-accent underline" @click="emit('transcript')">
          {{ t('transcript.title') }}
        </button>
      </p>
      <p
        v-if="
          !chat.status?.configured ||
          (chat.status.provider === 'deepseek' && !chat.status.has_key)
        "
        class="text-sm text-fg/60"
      >
        {{ t('chat.needsModel') }}
        <button
          class="text-accent underline"
          @click="settings.openSettings('ai')"
        >
          {{ t('summary.openSettings') }}
        </button>
      </p>
      <p v-if="summaries.isRunning" role="status" class="text-sm text-fg/60">
        {{ t('chat.summaryBusy') }}
      </p>
      <p
        v-if="chat.errorCode"
        :role="chat.errorCode === 'cancelled' ? 'status' : 'alert'"
        class="text-sm text-danger"
      >
        {{ t(`chat.error.${chat.errorCode}`) }}
        <button
          v-if="chat.messages.length"
          data-testid="chat-retry"
          :disabled="chat.isRunning || chat.dictationBusy"
          class="text-accent underline ml-2 disabled:opacity-40"
          @click="chat.retry()"
        >
          {{ t('chat.retry') }}
        </button>
      </p>
      <p v-if="chat.draftTooLong" role="alert" class="text-sm text-danger">
        {{ t('chat.tooLong') }}
      </p>
      <div class="flex items-center gap-2 text-xs text-fg/60">
        <FileAudio class="w-4 h-4" aria-hidden="true" /><span
          class="truncate"
          >{{ player.mediaInfo?.file_name }}</span
        ><span v-if="chat.status" class="ml-auto truncate font-mono"
          >{{ chat.status.model }} ·
          {{ providerHost(chat.status.base_url) }}</span
        >
      </div>
      <ChatComposer
        ref="composer"
        :model-value="chat.draft"
        :disabled="!chat.canSend"
        :sending="chat.isRunning"
        :remote-host="
          chat.status?.remote ? providerHost(chat.status.base_url) : null
        "
        @update:model-value="chat.setDraft"
        @send="chat.send()"
        @cancel="chat.cancel()"
        ><template #dictation><slot name="dictation"><DictationControl @focus="composer?.focus()" /></slot></template
      ></ChatComposer>
    </template>
    <p class="text-xs text-fg/60 leading-relaxed">
      {{ t('chat.sessionOnly') }}
    </p>
    <SummaryPromptEditor
      :open="promptOpen"
      :initial-draft="prompt"
      @close="promptOpen = false"
    />
  </section>
</template>
<script setup lang="ts">
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue';
import { AudioLines, FileAudio, X } from '@lucide/vue';
import ChatMessage from './ChatMessage.vue';
import ChatComposer from './ChatComposer.vue';
import DictationControl from './DictationControl.vue';
import SummaryPromptEditor from '@/components/transcript/SummaryPromptEditor.vue';
import { useChatStore } from '@/stores/chatStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useSummaryStore } from '@/stores/summaryStore';
import { useI18n } from '@/composables/useI18n';
import { providerHost } from '@/utils/providerLocation';
const props = defineProps<{
  visible: boolean;
  scrollElement?: HTMLElement | null;
}>();
const emit = defineEmits<{
  (event: 'close'): void;
  (event: 'transcript'): void;
}>();
const { t } = useI18n();
const chat = useChatStore();
const player = usePlayerStore();
const settings = useSettingsStore();
const summaries = useSummaryStore();
const replacement = ref<string | null>(null);
const prompt = ref('');
const promptOpen = ref(false);
const heading = ref<HTMLElement | null>(null);
const composer = ref<InstanceType<typeof ChatComposer> | null>(null);
const unread = ref(false);
let atBottom = true;
const opened = new Set<string>();
const starters = [
  { id: 'actions', label: 'chat.starterActions', prompt: 'chat.askActions' },
  { id: 'unclear', label: 'chat.starterUnclear', prompt: 'chat.askUnclear' },
  { id: 'prompt', label: 'chat.starterPrompt', prompt: 'chat.askPrompt' },
] as const;
function choose(text: string) {
  if (chat.draft.trim() && chat.draft !== text) replacement.value = text;
  else {
    chat.setDraft(text);
    composer.value?.focus();
  }
}
function replace() {
  if (replacement.value !== null) chat.setDraft(replacement.value);
  replacement.value = null;
  composer.value?.focus();
}
function reviewPrompt(text: string) {
  if (!promptOpen.value) {
    prompt.value = text;
    promptOpen.value = true;
  }
}
function latest() {
  const element = props.scrollElement;
  if (element) element.scrollTop = element.scrollHeight;
  atBottom = true;
  unread.value = false;
}
function measure() {
  const el = props.scrollElement;
  if (el) atBottom = el.scrollHeight - el.clientHeight - el.scrollTop <= 48;
  if (atBottom) unread.value = false;
}
watch(
  () => props.scrollElement,
  (element, previous) => {
    previous?.removeEventListener('scroll', measure);
    element?.addEventListener('scroll', measure, { passive: true });
    measure();
  },
  { immediate: true },
);
onScopeDispose(() =>
  props.scrollElement?.removeEventListener('scroll', measure),
);
watch(
  () => [props.visible, chat.path] as const,
  async ([visible, path]) => {
    if (!visible || !path || opened.has(path)) return;
    opened.add(path);
    await nextTick();
    heading.value?.scrollIntoView({ block: 'start' });
  },
  { immediate: true },
);
const lastUpdate = computed(() => {
  const last = chat.messages[chat.messages.length - 1];
  return `${chat.path}:${chat.messages.length}:${last?.text.length}:${last?.status}`;
});
watch(lastUpdate, async () => {
  const follow = atBottom;
  await nextTick();
  if (!props.visible) return;
  if (follow) latest();
  else unread.value = true;
});
watch(
  () => chat.path,
  () => {
    replacement.value = null;
    promptOpen.value = false;
    unread.value = false;
  },
);
defineExpose({ focusComposer: () => composer.value?.focus() });
</script>
