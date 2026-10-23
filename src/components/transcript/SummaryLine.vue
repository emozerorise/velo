<template>
  <div class="whitespace-pre-wrap break-words">
    <template v-for="(part, index) in prose" :key="index">
      <strong v-if="part.type === 'bold'" class="font-semibold text-fg/95">{{ part.text }}</strong>
      <template v-else>{{ part.text }}</template>
    </template>
  </div>
  <div v-if="showTimestamps && citations.length" class="flex flex-wrap gap-1.5 mt-2 select-none">
    <button
      v-for="(part, index) in citations"
      :key="index"
      class="inline-flex items-center gap-1 rounded-md border border-accent/15 bg-accent/[0.07] px-2 py-0.5 font-mono text-[11px] leading-5 text-accent hover:bg-accent/15 focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent transition-colors"
      :title="t('summary.jump', { time: part.label })"
      :aria-label="t('summary.jump', { time: part.label })"
      @click="emit('seek', part.seconds)"
    >
      <Play class="w-2.5 h-2.5" aria-hidden="true" />
      {{ part.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
import type { InlinePart } from '@/utils/summaryMarkdown';
import { computed } from 'vue';
import { Play } from '@lucide/vue';
import { useI18n } from '@/composables/useI18n';

const props = withDefaults(defineProps<{ parts: InlinePart[]; showTimestamps?: boolean }>(), {
  showTimestamps: false,
});
const prose = computed(() => props.parts.filter((part) => part.type !== 'stamp'));
const citations = computed(() => props.parts.filter((part) => part.type === 'stamp'));
const emit = defineEmits<{ (e: 'seek', seconds: number): void }>();

const { t } = useI18n();
</script>
