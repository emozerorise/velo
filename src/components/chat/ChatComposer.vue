<template>
  <div
    class="rounded-3xl border border-accent/75 focus-within:ring-2 focus-within:ring-accent bg-fg/[0.025] p-4 sm:p-5 shadow-[0_4px_24px_rgb(0_0_0/0.06)]"
  >
    <textarea
      ref="field"
      data-testid="chat-draft"
      :value="modelValue"
      :aria-label="t('chat.draftLabel')"
      :placeholder="t('chat.placeholder')"
      rows="3"
      class="block w-full min-h-24 max-h-64 resize-y bg-transparent text-base leading-relaxed text-fg/90 placeholder:text-fg/60 focus:outline-none"
      @input="
        emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)
      "
      @compositionstart="composing = true"
      @compositionend="composing = false"
      @keydown.stop="keyDown"
    />
    <div class="flex flex-wrap items-center gap-3 mt-3">
      <slot name="dictation" />
      <p class="flex-1 min-w-32 text-xs leading-relaxed text-fg/65">
        {{
          remoteHost ? t('chat.remote', { host: remoteHost }) : t('chat.local')
        }}
      </p>
      <button
        v-if="sending"
        type="button"
        data-testid="chat-stop"
        :aria-label="t('chat.stop')"
        class="w-11 h-11 shrink-0 rounded-full border border-fg/20 grid place-items-center text-fg hover:bg-fg/10 focus-visible:outline focus-visible:outline-accent"
        @click="emit('cancel')"
      >
        <Square class="w-4 h-4 fill-current" aria-hidden="true" />
      </button>
      <button
        v-else
        type="button"
        data-testid="chat-send"
        :aria-label="t('chat.send')"
        :disabled="disabled"
        class="w-11 h-11 shrink-0 rounded-full bg-blue-600 text-white grid place-items-center disabled:opacity-30 hover:brightness-110 focus-visible:outline focus-visible:outline-accent focus-visible:outline-offset-2"
        @click="send"
      >
        <ArrowUp class="w-5 h-5" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue';
import { ArrowUp, Square } from '@lucide/vue';
import { useI18n } from '@/composables/useI18n';
const props = defineProps<{
  modelValue: string;
  disabled: boolean;
  sending: boolean;
  remoteHost: string | null;
}>();
const emit = defineEmits<{
  (event: 'update:modelValue', value: string): void;
  (event: 'send'): void;
  (event: 'cancel'): void;
}>();
const { t } = useI18n();
const composing = ref(false);
const field = ref<HTMLTextAreaElement | null>(null);
function send() {
  if (!props.disabled && !props.sending && props.modelValue.trim())
    emit('send');
}
function keyDown(event: KeyboardEvent) {
  if (
    event.key === 'Enter' &&
    !event.shiftKey &&
    !event.isComposing &&
    !composing.value &&
    event.keyCode !== 229
  ) {
    event.preventDefault();
    send();
  }
}
defineExpose({ focus: () => field.value?.focus() });
</script>
