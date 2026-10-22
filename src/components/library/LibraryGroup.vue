<template>
  <section class="mb-3">
    <div class="flex items-center gap-1 px-2 py-1.5">
      <button class="flex items-center gap-1 min-w-0 flex-1 text-left text-xs font-semibold text-fg/60" :aria-expanded="!group.collapsed" :disabled="searching || !library.writesEnabled" @click="collapse">
        <ChevronRight class="w-3.5 h-3.5 shrink-0 transition-transform" :class="{'rotate-90':!group.collapsed}" />
        <span class="truncate">{{ group.name || t('library.uncategorized') }}</span><span class="ml-auto tabular-nums text-fg/40">{{ group.matching }}/{{ group.total }}</span>
      </button>
      <button ref="trigger" class="p-1 text-fg/45" :aria-label="`${t('library.actions')}: ${group.name || t('library.uncategorized')}`" :aria-expanded="menu" :disabled="!library.writesEnabled" @click="menu=!menu"><Ellipsis class="w-4 h-4" /></button>
    </div>
    <div v-if="menu" class="px-3 py-2 text-xs space-y-2 bg-fg/5 rounded-lg" @keydown.esc.stop.prevent="closeMenu">
      <button class="block text-accent" @click="openFileDialog(group.id === 'uncategorized' ? null : group.id)">{{ t('library.addHere') }}</button>
      <template v-if="group.id !== 'uncategorized'">
        <button class="block" @click="startRename">{{ t('library.renameProject') }}</button>
        <button class="block text-danger" @click="confirm=true">{{ t('library.deleteProject') }}</button>
        <div v-if="confirm"><p class="mb-2">{{ t('library.deleteHint') }}</p><button class="text-danger mr-3" @click="remove">{{ t('library.confirmRemove') }}</button><button @click="confirm=false">{{ t('library.cancel') }}</button></div>
      </template>
    </div>
    <LibraryNameEditor v-if="editing" :initial="group.name" :label="t('library.renameProject')" :save="rename" @saved="editing=false" @cancel="editing=false" />
    <div v-if="!group.collapsed" class="space-y-0.5"><LibraryRow v-for="item in group.items" :key="item.id" :item="item" /></div>
  </section>
</template>
<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { ChevronRight, Ellipsis } from '@lucide/vue';
import { useLibraryStore } from '@/stores/libraryStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useI18n } from '@/composables/useI18n';
import type { visibleLibrary } from '@/utils/libraryView';
import LibraryRow from './LibraryRow.vue';
import LibraryNameEditor from './LibraryNameEditor.vue';
const props = defineProps<{group: ReturnType<typeof visibleLibrary>['groups'][number]; searching?: boolean}>();
const library = useLibraryStore(), { t } = useI18n(), { openFileDialog } = useKeyboardShortcuts();
const menu = ref(false), editing = ref(false), confirm = ref(false);
const trigger = ref<HTMLButtonElement>();
function startRename(event:MouseEvent) {(event.currentTarget as HTMLButtonElement).focus();editing.value=true;}
async function closeMenu() { menu.value=false;confirm.value=false;await nextTick();trigger.value?.focus(); }
function collapse() {
  if (!props.searching) void library.toggleGroup(props.group.id);
}
function rename(name:string) { return library.apply({type:'rename_project',id:props.group.id,name}); }
async function remove() { if(await library.apply({type:'delete_project',id:props.group.id})) {menu.value=false;confirm.value=false;} }
</script>
