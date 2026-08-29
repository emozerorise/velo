<template>
  <BaseModal
    :is-open="settingsStore.isMediaInfoOpen"
    title="Media Information"
    @close="settingsStore.isMediaInfoOpen = false"
  >
    <div v-if="playerStore.mediaInfo" class="flex flex-col gap-4 text-xs font-mono">
      <!-- Grid items -->
      <div class="grid grid-cols-2 gap-3 p-4 bg-black/40 rounded-xl border border-white/5">
        <div>
          <span class="text-white/40 block text-[10px] uppercase font-sans">Resolution</span>
          <span class="text-white/90 text-sm font-semibold">
            {{ playerStore.mediaInfo.width }} x {{ playerStore.mediaInfo.height }}
          </span>
        </div>

        <div>
          <span class="text-white/40 block text-[10px] uppercase font-sans">Hardware Decoder</span>
          <span class="text-emerald-400 text-sm font-semibold">
            {{ playerStore.mediaInfo.hwdec_current || 'Active (Hardware)' }}
          </span>
        </div>

        <div>
          <span class="text-white/40 block text-[10px] uppercase font-sans">Video Codec</span>
          <span class="text-white/90">{{ playerStore.mediaInfo.video_codec || 'N/A' }}</span>
        </div>

        <div>
          <span class="text-white/40 block text-[10px] uppercase font-sans">Audio Codec</span>
          <span class="text-white/90">{{ playerStore.mediaInfo.audio_codec || 'N/A' }}</span>
        </div>

        <div>
          <span class="text-white/40 block text-[10px] uppercase font-sans">Framerate</span>
          <span class="text-white/90">{{ Math.round(playerStore.mediaInfo.fps * 100) / 100 }} FPS</span>
        </div>

        <div>
          <span class="text-white/40 block text-[10px] uppercase font-sans">Duration</span>
          <span class="text-white/90">{{ formatTime(playerStore.mediaInfo.duration) }}</span>
        </div>
      </div>

      <!-- File Path -->
      <div class="p-3 bg-black/40 rounded-xl border border-white/5 break-all">
        <span class="text-white/40 block text-[10px] uppercase font-sans mb-1">File Location</span>
        <span class="text-white/70">{{ playerStore.mediaInfo.path }}</span>
      </div>
    </div>

    <div v-else class="py-8 text-center text-white/40 text-sm italic">
      No media file currently loaded
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import BaseModal from '@/components/common/BaseModal.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { formatTime } from '@/utils/formatters';

const playerStore = usePlayerStore();
const settingsStore = useSettingsStore();
</script>
