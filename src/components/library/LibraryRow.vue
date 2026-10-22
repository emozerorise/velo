<template>
  <div class="rounded-xl border border-transparent" :class="active ? 'bg-accent/10 border-accent/20' : 'hover:bg-fg/5'">
    <div class="flex items-center min-w-0">
      <button class="flex gap-2 items-center min-w-0 flex-1 text-left px-2 py-2.5" :aria-current="active ? 'true' : undefined" :title="`${recordingName(item)}\n${item.path}`" @click="play">
        <Play v-if="active" class="w-4 h-4 text-accent shrink-0" /><FileAudio v-else class="w-4 h-4 text-fg/40 shrink-0" />
        <span class="min-w-0"><span class="block truncate text-sm text-fg/90">{{ recordingName(item) }}</span>
          <span class="block truncate text-xs text-fg/45 mt-0.5">{{ item.duration != null ? formatTime(item.duration) : '' }}<span v-if="item.display_name">{{ item.duration != null ? ' · ' : '' }}{{ baseName(item.path) }}</span></span>
        </span>
      </button>
      <button ref="trigger" class="p-2 text-fg/45 hover:text-fg" :aria-label="`${t('library.actions')}: ${recordingName(item)}`" :aria-expanded="menu" :disabled="!library.writesEnabled" @click="menu=!menu"><Ellipsis class="w-4 h-4" /></button>
    </div>
    <p v-if="library.unavailable.includes(item.path)" class="px-2 pb-2 text-xs text-danger">{{ t('library.missing') }}</p>
    <LibraryItemMenu v-if="menu" :item="item" @rename="rename" @done="close" @keydown.esc.stop="close" />
  </div>
</template>
<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { Ellipsis, FileAudio, Play } from '@lucide/vue';
import { baseName, type LibraryRecording } from '@/types/library';
import { recordingName } from '@/utils/libraryView';
import { useLibraryStore } from '@/stores/libraryStore';
import { usePlayerStore } from '@/stores/playerStore';
import { playLibraryRecording } from '@/services/mediaIngestion';
import { formatTime } from '@/utils/formatters';
import { useI18n } from '@/composables/useI18n';
import LibraryItemMenu from './LibraryItemMenu.vue';
const props = defineProps<{item: LibraryRecording}>();
const library = useLibraryStore(), player = usePlayerStore(), { t } = useI18n();
const menu = ref(false), trigger = ref<HTMLButtonElement>();
const active = computed(() => player.mediaInfo?.path === props.item.path);
async function close() { menu.value=false;await nextTick();trigger.value?.focus(); }
function rename() {
  // The menu item disappears; restore to the persistent row control instead.
  trigger.value?.focus();
  library.beginRename(props.item.id);
  menu.value=false;
}
async function play() { try { await playLibraryRecording(props.item.path); } catch { library.error=t('library.playError'); } }
</script>
