<template>
  <div class="flex flex-wrap items-center gap-2 text-xs">
    <button
      v-if="updates.phase === 'available' || updates.phase === 'downloaded'"
      class="rounded-lg bg-blue-600 px-3 py-2 text-white hover:bg-blue-500 disabled:opacity-40"
      :disabled="updates.workBusy"
      @click="updates.downloadAndInstall()"
    >
      {{ t(updates.phase === 'downloaded' ? 'updates.install' : 'updates.download') }}
    </button>
    <button
      v-if="updates.phase === 'ready'"
      class="rounded-lg bg-blue-600 px-3 py-2 text-white hover:bg-blue-500 disabled:opacity-40"
      :disabled="updates.workBusy"
      @click="updates.restart()"
    >
      {{ t('updates.restart') }}
    </button>
    <span v-if="updates.phase === 'downloading'" role="status">
      {{ t('updates.downloading') }}
      {{ updates.progress < 0 ? formatBytes(updates.downloaded) : `${Math.round(updates.progress * 100)}%` }}
    </span>
    <button
      v-if="['available', 'downloaded', 'ready', 'downloading'].includes(updates.phase)"
      class="rounded-lg border border-fg/20 px-3 py-2 hover:bg-fg/10"
      @click="updates.dismissed = true"
    >
      {{ t('updates.later') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useUpdateStore } from '@/stores/updateStore';
import { useI18n } from '@/composables/useI18n';
import { formatBytes } from '@/utils/formatters';
const updates = useUpdateStore();
const { t } = useI18n();
</script>
