<template>
  <div ref="rootRef" class="relative">
    <IconButton
      :title="t('controls.subtitles')"
      size="md"
      :active="isOpen || hasActiveSubtitle"
      @click="toggle"
    >
      <Subtitles class="w-5 h-5" />
    </IconButton>

    <!-- Dropdown Menu -->
    <div
      v-if="isOpen"
      class="absolute bottom-12 right-0 w-64 p-2 bg-surface border border-fg/10 rounded-xl z-50 flex flex-col gap-1 text-sm text-fg"
    >
      <div class="px-3 py-1.5 text-xs font-semibold text-fg/40 uppercase tracking-wider border-b border-fg/10 flex items-center justify-between">
        <span>{{ t('controls.subtitles') }}</span>
        <button
          class="text-[10px] text-accent hover:underline uppercase tracking-normal"
          @click="addExternalSubtitle"
        >
          {{ t('controls.addSubtitleFile') }}
        </button>
      </div>

      <div class="max-h-56 overflow-y-auto flex flex-col gap-0.5">
        <!-- Disable Subtitles Option -->
        <button
          :class="[
            'w-full px-3 py-2 text-left rounded-lg text-xs flex items-center justify-between transition-colors',
            !hasActiveSubtitle
              ? 'bg-blue-500/15 text-accent font-medium'
              : 'text-fg/80 hover:bg-fg/10',
          ]"
          @click="selectTrack(0)"
        >
          <span>{{ t('controls.subtitlesOff') }}</span>
          <Check v-if="!hasActiveSubtitle" class="w-3.5 h-3.5 flex-shrink-0" />
        </button>

        <!-- Subtitle Tracks -->
        <button
          v-for="track in playerStore.subtitleTracks"
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
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDismissable } from '@/composables/useDismissable';
import { Subtitles, Check } from '@lucide/vue';
import { open } from '@tauri-apps/plugin-dialog';
import IconButton from '@/components/common/IconButton.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { formatTrackLabel } from '@/utils/formatters';
import { useI18n } from '@/composables/useI18n';

const playerStore = usePlayerStore();
const { t } = useI18n();
const { isOpen, rootRef, toggle, close } = useDismissable();

const hasActiveSubtitle = computed(() => {
  return playerStore.subtitleTracks.some((t) => t.selected);
});

function selectTrack(id: number) {
  playerStore.selectSubtitleTrack(id);
  close();
}

async function addExternalSubtitle() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: t('dialog.subtitleFiles'),
          extensions: ['srt', 'ass', 'ssa', 'vtt', 'sub'],
        },
      ],
    });

    if (selected && typeof selected === 'string') {
      await playerStore.addSubtitleFile(selected);
      close();
    }
  } catch (e) {
    console.error('Failed to open subtitle dialog:', e);
  }
}
</script>
