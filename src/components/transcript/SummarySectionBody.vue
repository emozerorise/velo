<template>
  <div class="summary-body reading-text space-y-4 text-fg/80 select-text">
    <div
      v-for="(node, index) in nodes"
      :key="index"
      class="flex gap-2.5 min-w-0"
      :style="node.type === 'bullet' ? { paddingLeft: `${node.depth * 0.9}rem` } : undefined"
    >
      <span
        v-if="node.type === 'bullet'"
        class="summary-bullet mt-[0.8em] h-1 w-1 rounded-full bg-fg/30 shrink-0"
        aria-hidden="true"
      />
      <div class="min-w-0 flex-1">
        <component
          :is="`h${Math.min(3 + node.depth, 6)}`"
          v-if="node.type === 'subheading'"
          class="font-semibold text-fg/90"
        >{{ node.text }}</component>
        <SummaryLine
          v-else
          :parts="node.parts"
          :show-timestamps="showTimestamps"
          @seek="emit('seek', $event)"
        />
      </div>
    </div>
    <p v-if="!nodes.length" class="text-sm text-fg/40">{{ t('summary.notSpecified') }}</p>
  </div>
</template>

<script setup lang="ts">
import SummaryLine from './SummaryLine.vue';
import type { SummarySection } from '@/utils/summaryMarkdown';
import { useI18n } from '@/composables/useI18n';

withDefaults(defineProps<{ nodes: SummarySection['nodes']; showTimestamps?: boolean }>(), {
  showTimestamps: true,
});
const emit = defineEmits<{ (e: 'seek', seconds: number): void }>();
const { t } = useI18n();
</script>
