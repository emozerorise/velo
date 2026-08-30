<template>
  <div
    :class="[
      'group relative flex items-center justify-between px-3 py-2.5 rounded-xl text-xs transition-all duration-150 cursor-pointer',
      isActive
        ? 'bg-blue-600/25 border border-blue-500/40 text-accent font-medium'
        : 'text-fg/80 hover:bg-fg/10 hover:text-fg',
    ]"
    @click="$emit('play')"
  >
    <div class="flex items-center gap-2.5 truncate">
      <!-- Play Indicator / Index -->
      <span class="w-4 text-[10px] text-fg/40 font-mono text-center flex-shrink-0">
        {{ index + 1 }}
      </span>

      <!-- Title -->
      <span class="truncate">{{ item.fileName }}</span>
    </div>

    <!-- Actions (Delete) -->
    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
      <button
        class="p-1 rounded text-fg/40 hover:text-danger hover:bg-fg/10 transition-colors"
        :title="t('playlist.remove')"
        @click.stop="$emit('remove')"
      >
        <Trash2 class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Trash2 } from '@lucide/vue';
import type { PlaylistItem } from '@/types/playlist';
import { useI18n } from '@/composables/useI18n';

const { t } = useI18n();

defineProps<{
  item: PlaylistItem;
  index: number;
  isActive: boolean;
}>();

defineEmits<{
  (e: 'play'): void;
  (e: 'remove'): void;
}>();
</script>
