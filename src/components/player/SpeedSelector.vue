<template>
  <div ref="rootRef" class="relative">
    <button
      class="px-2.5 py-1.5 rounded-lg text-xs font-mono tabular-nums font-medium text-white/75 hover:text-white hover:bg-white/10 transition-colors no-drag"
      :title="'Playback Speed: ' + playerStore.speed + 'x'"
      @click="toggle"
    >
      {{ playerStore.speed }}x
    </button>

    <!-- Dropdown Menu -->
    <div
      v-if="isOpen"
      class="absolute bottom-12 right-0 w-36 p-1.5 bg-[#15151b] border border-white/10 rounded-xl z-50 flex flex-col gap-0.5 text-xs text-white"
    >
      <div class="px-2 py-1 text-[10px] font-semibold text-white/40 uppercase tracking-wider border-b border-white/10">
        Speed
      </div>

      <button
        v-for="preset in speedPresets"
        :key="preset"
        :class="[
          'w-full px-2.5 py-1.5 text-left rounded-md flex items-center justify-between font-mono transition-colors',
          playerStore.speed === preset
            ? 'bg-blue-500/15 text-blue-400 font-semibold'
            : 'text-white/80 hover:bg-white/10',
        ]"
        @click="selectSpeed(preset)"
      >
        <span>{{ preset }}x</span>
        <Check v-if="playerStore.speed === preset" class="w-3 h-3" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDismissable } from '@/composables/useDismissable';
import { Check } from '@lucide/vue';
import { usePlayerStore } from '@/stores/playerStore';

const playerStore = usePlayerStore();
const { isOpen, rootRef, toggle, close } = useDismissable();

const speedPresets = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

function selectSpeed(s: number) {
  playerStore.setSpeed(s);
  close();
}
</script>
