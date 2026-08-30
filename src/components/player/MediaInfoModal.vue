<template>
  <BaseModal
    :is-open="settingsStore.isMediaInfoOpen"
    :title="t('mediaInfo.title')"
    @close="settingsStore.isMediaInfoOpen = false"
  >
    <div v-if="playerStore.mediaInfo" class="flex flex-col gap-4 text-xs font-mono">
      <!-- Grid items -->
      <div class="grid grid-cols-2 gap-3 p-4 bg-inset/40 rounded-xl border border-fg/5">
        <div>
          <span class="text-fg/40 block text-[10px] uppercase font-sans">{{ t('mediaInfo.resolution') }}</span>
          <span class="text-fg/90 text-sm font-semibold">
            {{ playerStore.mediaInfo.width }} x {{ playerStore.mediaInfo.height }}
          </span>
        </div>

        <div>
          <span class="text-fg/40 block text-[10px] uppercase font-sans">{{ t('mediaInfo.hwdec') }}</span>
          <span class="text-success text-sm font-semibold">
            {{ playerStore.mediaInfo.hwdec_current || t('mediaInfo.hwdecActive') }}
          </span>
        </div>

        <div>
          <span class="text-fg/40 block text-[10px] uppercase font-sans">{{ t('mediaInfo.videoCodec') }}</span>
          <span class="text-fg/90">{{ playerStore.mediaInfo.video_codec || t('mediaInfo.na') }}</span>
        </div>

        <div>
          <span class="text-fg/40 block text-[10px] uppercase font-sans">{{ t('mediaInfo.audioCodec') }}</span>
          <span class="text-fg/90">{{ playerStore.mediaInfo.audio_codec || t('mediaInfo.na') }}</span>
        </div>

        <div>
          <span class="text-fg/40 block text-[10px] uppercase font-sans">{{ t('mediaInfo.framerate') }}</span>
          <span class="text-fg/90">{{ Math.round(playerStore.mediaInfo.fps * 100) / 100 }} FPS</span>
        </div>

        <div>
          <span class="text-fg/40 block text-[10px] uppercase font-sans">{{ t('mediaInfo.duration') }}</span>
          <span class="text-fg/90">{{ formatTime(playerStore.mediaInfo.duration) }}</span>
        </div>
      </div>

      <!-- File Path -->
      <div class="p-3 bg-inset/40 rounded-xl border border-fg/5 break-all">
        <span class="text-fg/40 block text-[10px] uppercase font-sans mb-1">{{ t('mediaInfo.filePath') }}</span>
        <span class="text-fg/70">{{ playerStore.mediaInfo.path }}</span>
      </div>
    </div>

    <div v-else class="py-8 text-center text-fg/40 text-sm italic">
      {{ t('mediaInfo.none') }}
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import BaseModal from '@/components/common/BaseModal.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
import { formatTime } from '@/utils/formatters';

const playerStore = usePlayerStore();
const { t } = useI18n();
const settingsStore = useSettingsStore();
</script>
