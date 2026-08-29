<template>
  <div class="w-full max-w-5xl mx-auto px-4 pb-4 pointer-events-auto">
    <!-- Fully opaque. A gradient or blurred panel would veil the video, which
         is a native window behind this one. -->
    <div
      class="rounded-2xl bg-[#0e0e12] border border-white/10 px-4 pt-2 pb-3 flex flex-col gap-1.5"
    >
      <TimelineBar />

      <div class="flex items-center justify-between gap-4">
        <!-- Playback -->
        <div class="flex items-center gap-1">
          <IconButton
            title="Previous"
            size="md"
            :disabled="!playlistStore.hasPrevious"
            @click="playlistStore.previous()"
          >
            <SkipBack class="w-[18px] h-[18px]" />
          </IconButton>

          <button
            class="w-11 h-11 mx-1 rounded-full bg-blue-500 hover:bg-blue-400 active:scale-95 text-white flex items-center justify-center transition-all no-drag"
            :title="isPlaying ? 'Pause (Space)' : 'Play (Space)'"
            @click="playerStore.togglePlay()"
          >
            <Pause v-if="isPlaying" class="w-[18px] h-[18px] fill-current" />
            <Play v-else class="w-[18px] h-[18px] fill-current translate-x-[1px]" />
          </button>

          <IconButton
            title="Next"
            size="md"
            :disabled="!playlistStore.hasNext"
            @click="playlistStore.next()"
          >
            <SkipForward class="w-[18px] h-[18px]" />
          </IconButton>

          <!-- Volume -->
          <div class="flex items-center gap-2 ml-2">
            <IconButton
              :title="playerStore.muted ? 'Unmute (M)' : 'Mute (M)'"
              size="md"
              @click="playerStore.toggleMute()"
            >
              <VolumeX v-if="isSilent" class="w-[18px] h-[18px]" />
              <Volume1 v-else-if="playerStore.volume < 50" class="w-[18px] h-[18px]" />
              <Volume2 v-else class="w-[18px] h-[18px]" />
            </IconButton>

            <input
              type="range"
              min="0"
              max="100"
              step="1"
              class="velo-range w-20"
              :value="volumeValue"
              :style="volumeTrackStyle"
              @input="onVolumeInput"
            />
          </div>

          <!-- Time -->
          <div class="ml-3 flex items-baseline gap-1.5 text-[13px] font-mono tabular-nums">
            <span class="text-white">{{ formatTime(displayTime) }}</span>
            <span class="text-white/30">/</span>
            <span class="text-white/45">{{ formatTime(playerStore.duration) }}</span>
          </div>
        </div>

        <!-- Tracks & view -->
        <div class="flex items-center gap-1">
          <AudioTrackSelector />
          <SubtitleTrackSelector />
          <SpeedSelector />

          <IconButton
            :title="playerStore.isFullscreen ? 'Exit Fullscreen (F)' : 'Fullscreen (F)'"
            size="md"
            @click="toggleFullscreen"
          >
            <Minimize v-if="playerStore.isFullscreen" class="w-[18px] h-[18px]" />
            <Maximize v-else class="w-[18px] h-[18px]" />
          </IconButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Volume1,
  Volume2,
  VolumeX,
  Maximize,
  Minimize,
} from '@lucide/vue';
import IconButton from '@/components/common/IconButton.vue';
import TimelineBar from '@/components/player/TimelineBar.vue';
import AudioTrackSelector from '@/components/player/AudioTrackSelector.vue';
import SubtitleTrackSelector from '@/components/player/SubtitleTrackSelector.vue';
import SpeedSelector from '@/components/player/SpeedSelector.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { formatTime } from '@/utils/formatters';

const playerStore = usePlayerStore();
const playlistStore = usePlaylistStore();
const { toggleFullscreen } = useKeyboardShortcuts();

const isPlaying = computed(() => playerStore.state === 'playing');
const isSilent = computed(() => playerStore.muted || playerStore.volume === 0);
const volumeValue = computed(() => (playerStore.muted ? 0 : playerStore.volume));

const displayTime = computed(() => playerStore.seekPreview ?? playerStore.currentTime);

const volumeTrackStyle = computed(() => ({
  background: `linear-gradient(to right, #fff ${volumeValue.value}%, rgba(255,255,255,0.25) ${volumeValue.value}%)`,
}));

function onVolumeInput(e: Event) {
  const target = e.target as HTMLInputElement;
  const val = parseFloat(target.value);
  playerStore.setVolume(val);
  if (playerStore.muted && val > 0) {
    playerStore.toggleMute();
  }
}
</script>
