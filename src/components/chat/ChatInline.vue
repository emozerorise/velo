<template>
  <template v-for="(part, index) in parts" :key="index">
    <strong v-if="part.type === 'strong'" class="font-semibold text-fg/95"
      ><ChatInline :parts="part.parts" @seek="emit('seek', $event)"
    /></strong>
    <button
      v-else-if="part.type === 'citation'"
      type="button"
      class="chat-citation"
      :aria-label="t('summary.jump', { time: part.label })"
      @click="emit('seek', part.seconds)"
    >
      {{ part.label }}
    </button>
    <span v-else>{{ part.text }}</span>
  </template>
</template>
<script setup lang="ts">
import type { ChatPart } from "@/utils/chatContent";
import { useI18n } from "@/composables/useI18n";
defineProps<{ parts: ChatPart[] }>();
const emit = defineEmits<{ (event: "seek", seconds: number): void }>();
const { t } = useI18n();
</script>
<style scoped>
.chat-citation {
  display: inline-block;
  margin: 0 3px;
  padding: 2px 7px;
  border: 1px solid rgb(var(--velo-accent) / 0.16);
  border-radius: 7px;
  color: rgb(var(--velo-accent));
  background: rgb(var(--velo-accent) / 0.08);
  font:
    12px/1.5 ui-monospace,
    monospace;
  vertical-align: baseline;
}
.chat-citation:hover {
  background: rgb(var(--velo-accent) / 0.14);
}
.chat-citation:focus-visible {
  outline: 2px solid rgb(var(--velo-accent));
  outline-offset: 2px;
}
</style>
