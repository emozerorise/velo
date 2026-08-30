<template>
  <div ref="rootRef" class="relative">
    <IconButton
      :title="t('controls.audioTracks')"
      size="md"
      :active="isOpen"
      @click="toggle"
    >
      <Volume2 class="w-5 h-5" />
    </IconButton>

    <!-- Dropdown Menu -->
    <div
      v-if="isOpen"
      class="absolute bottom-12 right-0 w-64 p-2 bg-surface border border-fg/10 rounded-xl z-50 flex flex-col gap-1 text-sm text-fg"
    >
      <div class="px-3 py-1.5 text-xs font-semibold text-fg/40 uppercase tracking-wider border-b border-fg/10">
        {{ t('controls.audioTracks') }}
      </div>

      <div class="max-h-56 overflow-y-auto flex flex-col gap-0.5">
        <button
          v-for="track in playerStore.audioTracks"
          :key="track.id"
          :class="[
            'w-full px-3 py-2 text-left rounded-lg text-xs flex items-center justify-between transition-colors',
            track.selected
              ? 'bg-blue-500/15 text-accent font-medium'
              : 'text-fg/80 hover:bg-fg/10',
          ]"
          @click="selectTrack(track.id)"
        >
          <span class="truncate">{{ formatTrackLabel(track) }}</span>
          <Check v-if="track.selected" class="w-3.5 h-3.5 flex-shrink-0" />
        </button>

        <div
          v-if="playerStore.audioTracks.length === 0"
          class="px-3 py-3 text-xs text-fg/40 text-center italic"
        >
          {{ t('controls.noAudioTracks') }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDismissable } from '@/composables/useDismissable';
import { Volume2, Check } from '@lucide/vue';
import IconButton from '@/components/common/IconButton.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { formatTrackLabel } from '@/utils/formatters';
import { useI18n } from '@/composables/useI18n';

const playerStore = usePlayerStore();
const { t } = useI18n();
const { isOpen, rootRef, toggle, close } = useDismissable();

function selectTrack(id: number) {
  playerStore.selectAudioTrack(id);
  close();
}
</script>
