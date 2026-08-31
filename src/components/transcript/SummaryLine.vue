<template>
  <template v-for="(part, index) in parts" :key="index">
    <button
      v-if="part.type === 'stamp'"
      class="font-mono text-[11px] text-accent/90 hover:text-accent hover:underline align-baseline"
      :title="t('summary.jump', { time: part.label })"
      @click="emit('seek', part.seconds)"
    >
      [{{ part.label }}]
    </button>
    <strong v-else-if="part.type === 'bold'" class="font-semibold text-fg/90">{{
      part.text
    }}</strong>
    <template v-else>{{ part.text }}</template>
  </template>
</template>

<script setup lang="ts">
import type { InlinePart } from "@/utils/summaryMarkdown";
import { useI18n } from "@/composables/useI18n";

defineProps<{ parts: InlinePart[] }>();
const emit = defineEmits<{ (e: "seek", seconds: number): void }>();

const { t } = useI18n();
</script>
