<template>
  <div
    ref="trackRef"
    class="group/track relative w-full h-5 flex items-center cursor-pointer"
    @mousedown="startDrag"
    @mousemove="handleHover"
    @mouseleave="hoverTime = null"
  >
    <!-- Track -->
    <div
      class="relative w-full h-[3px] group-hover/track:h-[5px] bg-fg/25 rounded-full overflow-hidden transition-[height] duration-150"
    >
      <!-- Hover scrub hint -->
      <div
        v-if="hoverTime !== null"
        class="absolute inset-y-0 left-0 bg-fg/25"
        :style="{ width: `${hoverPercent}%` }"
      />
      <!-- Played -->
      <div
        class="absolute inset-y-0 left-0 bg-blue-500 rounded-full"
        :style="{ width: `${displayPercent}%` }"
      />
    </div>

    <!-- Thumb -->
    <div
      class="absolute w-3 h-3 -translate-x-1/2 rounded-full bg-fg shadow-[0_1px_4px_rgba(0,0,0,0.6)] opacity-0 scale-50 group-hover/track:opacity-100 group-hover/track:scale-100 transition-all duration-150"
      :class="{ 'opacity-100 scale-110': isDragging }"
      :style="{ left: `${displayPercent}%` }"
    />

    <!-- Hover time -->
    <div
      v-if="hoverTime !== null"
      class="absolute -top-8 px-2 py-1 rounded-md bg-surface border border-fg/10 text-[11px] font-mono tabular-nums text-fg/90 -translate-x-1/2 pointer-events-none whitespace-nowrap"
      :style="{ left: `${hoverPercent}%` }"
    >
      {{ formatTime(hoverTime) }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { usePlayerStore } from '@/stores/playerStore';
import { formatTime } from '@/utils/formatters';

const playerStore = usePlayerStore();
const trackRef = ref<HTMLElement | null>(null);

const isDragging = ref(false);
const dragTime = ref(0);
const hoverTime = ref<number | null>(null);
const hoverPercent = ref(0);

const displayTime = computed(() => {
  if (isDragging.value) {
    return dragTime.value;
  }
  return playerStore.currentTime;
});

const displayPercent = computed(() => {
  if (playerStore.duration <= 0) return 0;
  return (displayTime.value / playerStore.duration) * 100;
});

function calculateTimeFromEvent(e: MouseEvent): number {
  if (!trackRef.value || playerStore.duration <= 0) return 0;
  const rect = trackRef.value.getBoundingClientRect();
  const clickX = e.clientX - rect.left;
  const ratio = Math.max(0, Math.min(1, clickX / rect.width));
  return ratio * playerStore.duration;
}

function handleHover(e: MouseEvent) {
  if (playerStore.duration <= 0) return;
  const time = calculateTimeFromEvent(e);
  hoverTime.value = time;
  hoverPercent.value = (time / playerStore.duration) * 100;
}

function startDrag(e: MouseEvent) {
  if (playerStore.duration <= 0) return;
  isDragging.value = true;
  playerStore.isDraggingTimeline = true;
  dragTime.value = calculateTimeFromEvent(e);
  playerStore.seekPreview = dragTime.value;

  const handleMouseMove = (moveEvent: MouseEvent) => {
    dragTime.value = calculateTimeFromEvent(moveEvent);
    playerStore.seekPreview = dragTime.value;
  };

  const handleMouseUp = () => {
    isDragging.value = false;
    playerStore.isDraggingTimeline = false;
    playerStore.seekAbsolute(dragTime.value);
    playerStore.seekPreview = null;
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  };

  window.addEventListener('mousemove', handleMouseMove);
  window.addEventListener('mouseup', handleMouseUp);
}
</script>
