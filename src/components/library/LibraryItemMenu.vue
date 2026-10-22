<template>
  <div class="flex flex-wrap gap-1 px-2 pb-2 text-xs" :aria-label="t('library.actions')">
    <button class="action" @click="emit('rename')">{{ t('library.rename') }}</button>
    <button class="action" @click="apply({type:'set_pin', id: item.id, pinned:!item.pinned})">{{ item.pinned ? t('library.unpin') : t('library.pin') }}</button>
    <button v-if="item.display_name" class="action" @click="apply({type:'reset_name', id:item.id})">{{ t('library.reset') }}</button>
    <label class="w-full text-fg/60">{{ t('library.move') }}
      <select class="w-full rounded border border-fg/15 bg-chrome p-1 text-fg" :value="item.project_id ?? ''" @change="move">
        <option value="">{{ t('library.uncategorized') }}</option>
        <option v-for="p in library.snapshot.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </label>
    <button v-if="!confirm" class="action text-danger" @click="confirm=true">{{ t('library.remove') }}</button>
    <div v-else class="space-y-2 p-2 border border-danger/20 rounded-lg">
      <p>{{ t('library.removeHint') }}</p>
      <button class="action text-danger" @click="remove">{{ t('library.confirmRemove') }}</button>
      <button class="action" @click="confirm=false">{{ t('library.cancel') }}</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue';
import type { LibraryRecording, LibraryMutation } from '@/types/library';
import { useLibraryStore } from '@/stores/libraryStore';
import { useI18n } from '@/composables/useI18n';
const props = defineProps<{item: LibraryRecording}>();
const emit = defineEmits<{rename: []; done: []}>();
const library = useLibraryStore(), { t } = useI18n(), confirm = ref(false);
async function apply(mutation: LibraryMutation) { if (await library.apply(mutation)) emit('done'); }
async function move(event: Event) {
  const select = event.target as HTMLSelectElement;
  const saved = await library.apply({type:'move_recording', id:props.item.id, project_id:select.value || null});
  if (saved) emit('done'); else select.value = props.item.project_id ?? '';
}
async function remove() { await apply({type:'remove_recording', id:props.item.id}); }
</script>
<style scoped>
.action { padding: 5px 7px; border-radius: 6px; }
.action:hover { background: rgb(var(--fg) / .06); }
</style>
