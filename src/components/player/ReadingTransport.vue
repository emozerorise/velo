<template>
  <footer class="shrink-0 bg-surface px-4 sm:px-6" :class="{ 'transport-compact': compact }" @keydown.space.stop>
    <div
      class="max-w-[68rem] mx-auto border-t border-fg/15 py-3 flex flex-wrap items-center gap-2 sm:gap-4"
    >
      <button
        class="p-2 rounded-lg text-fg/70 hover:text-fg hover:bg-fg/5"
        :aria-label="t('toast.seekBackward', { seconds: 5 })"
        @click="player.seek(-5, true)"
      >
        <RotateCcw class="w-5 h-5" /><span class="text-[10px]">5</span>
      </button>
      <button
        class="w-14 h-14 shrink-0 rounded-full bg-blue-500 hover:bg-blue-400 text-white flex items-center justify-center"
        :aria-label="player.state === 'playing' ? t('controls.pause') : t('controls.play')"
        @click="player.togglePlay()"
      >
        <Pause v-if="player.state === 'playing'" class="w-5 h-5 fill-current" /><Play
          v-else
          class="w-5 h-5 fill-current"
        />
      </button>
      <button
        class="p-2 rounded-lg text-fg/70 hover:text-fg hover:bg-fg/5"
        :aria-label="t('toast.seekForward', { seconds: 5 })"
        @click="player.seek(5, true)"
      >
        <RotateCw class="w-5 h-5" /><span class="text-[10px]">5</span>
      </button>
      <div class="flex-1 min-w-[80px]"><TimelineBar /></div>
      <span class="font-mono text-xs tabular-nums whitespace-nowrap text-fg/45"
        ><span class="text-accent">{{ formatTime(player.seekPreview ?? player.currentTime) }}</span>
        / {{ formatTime(player.duration) }}</span
      >
      <button
        class="p-2 rounded-lg text-fg/70 hover:text-fg"
        :aria-label="player.muted ? t('controls.unmute') : t('controls.mute')"
        @click="player.toggleMute()"
      >
        <VolumeX v-if="player.muted || player.volume === 0" class="w-4 h-4" /><Volume2
          v-else
          class="w-4 h-4"
        />
      </button>
      <input
        class="velo-range reading-volume w-20 hidden lg:block"
        type="range"
        min="0"
        max="100"
        :value="player.volume"
        :style="{
          background: `linear-gradient(to right, rgb(var(--velo-accent)) ${player.volume}%, rgb(var(--velo-fg) / 0.25) ${player.volume}%)`,
        }"
        :aria-label="t('workspace.volume')"
        @input="player.setVolume(Number(($event.target as HTMLInputElement).value))"
        @keydown.stop
      />
      <SpeedSelector />
    </div>
  </footer>
</template>

<script setup lang="ts">
import { Pause, Play, RotateCcw, RotateCw, Volume2, VolumeX } from '@lucide/vue';
import TimelineBar from './TimelineBar.vue';
import SpeedSelector from './SpeedSelector.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { useI18n } from '@/composables/useI18n';
import { formatTime } from '@/utils/formatters';
const player = usePlayerStore();
const { t } = useI18n();
defineProps<{ compact?: boolean }>();
</script>

<style scoped>
.transport-compact { padding: 0 12px; }
.transport-compact > div { padding: 8px 0; border: 0; gap: 8px; }
.transport-compact button.rounded-full { width: 40px; height: 40px; }
.reading-volume::-webkit-slider-thumb {
  opacity: 1;
  width: 10px;
  height: 10px;
  background: rgb(var(--velo-accent));
}
@media (max-height: 550px) {
  button.rounded-full {
    width: 44px;
    height: 44px;
  }
}
</style>
