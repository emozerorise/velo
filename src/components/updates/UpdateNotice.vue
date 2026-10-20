<template>
  <aside
    v-if="updates.noticeVisible && !updates.blocking"
    class="fixed bottom-32 left-4 right-4 z-[45] max-w-sm rounded-xl border border-fg/20 bg-surface/95 p-4 text-fg shadow-xl backdrop-blur-xl"
    aria-live="polite"
  >
    <div class="mb-3 flex items-center justify-between gap-3 text-sm">
      <span>{{ t(updates.phase === 'ready' ? 'updates.ready' : 'updates.available', { version: updates.version }) }}</span>
      <button class="text-blue-400 underline" @click="settings.openSettings('updates')">
        {{ t('updates.details') }}
      </button>
    </div>
    <p v-if="updates.workBusy" class="mb-3 text-xs text-fg/60">{{ t('updates.busy') }}</p>
    <p v-if="updates.error" class="mb-3 break-words text-xs text-red-400" role="alert">{{ updates.error }}</p>
    <UpdateActions />
  </aside>
</template>

<script setup lang="ts">
import { useUpdateStore } from '@/stores/updateStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
import UpdateActions from './UpdateActions.vue';
const updates = useUpdateStore();
const settings = useSettingsStore();
const { t } = useI18n();
</script>
