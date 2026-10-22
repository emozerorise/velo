<template>
  <div class="flex flex-col flex-1 min-h-0 border-t border-fg/5">
    <div class="px-4 pt-3 pb-2 space-y-3">
      <div class="flex items-center justify-between"><h2 class="text-sm font-semibold text-fg/85">{{ t('library.title') }} <span class="text-fg/40 font-normal">{{ library.snapshot.recordings.length }}</span></h2>
        <button class="text-accent p-1 rounded-md hover:bg-accent/10" :disabled="!library.writesEnabled" :aria-label="t('library.newProject')" :title="t('library.newProject')" @click="startCreate"><FolderPlus class="w-4 h-4" /></button></div>
      <label class="flex items-center gap-2 border border-fg/10 rounded-lg px-2 bg-fg/[0.03]"><Search class="w-4 h-4 text-fg/40" /><input v-model="query" type="search" class="min-w-0 flex-1 py-2 text-sm bg-transparent outline-none text-fg" :placeholder="t('library.search')" :aria-label="t('library.search')" /></label>
      <div class="flex items-center justify-between gap-1 text-xs text-fg/50"><span>{{ matching }}/{{ library.snapshot.recordings.length }}</span><select class="bg-chrome text-fg/65 text-xs rounded p-1 max-w-[170px]" :aria-label="t('library.sort')" :value="library.snapshot.preferences.sort" :disabled="!library.writesEnabled" @change="sort"><option value="added">{{ t('library.added') }}</option><option value="played">{{ t('library.played') }}</option><option value="name">{{ t('library.name') }}</option></select></div>
      <LibraryNameEditor v-if="creating" initial="" :label="t('library.newProject')" :save="create" @saved="creating=false" @cancel="creating=false" />
      <LibraryNameEditor v-if="library.renameTarget" :key="library.renameTarget.id" :initial="library.renameTarget.initial" :label="t('library.rename')" :save="rename" @saved="library.renameTarget=null" @cancel="library.renameTarget=null" />
      <p v-if="library.loading" class="text-xs text-fg/50">{{ t('library.loading') }}</p>
      <div v-if="library.error" role="alert" class="text-xs p-2 rounded-lg bg-danger/10 text-danger"><p>{{ t('library.error') }}</p><p class="mt-1 break-words">{{ library.error }}</p><button class="underline mt-1" @click="library.retryLoad()">{{ t('library.retry') }}</button></div>
    </div>
    <div class="flex-1 min-h-0 overflow-y-auto px-2 pb-3">
      <section v-if="view.pinned.length" class="mb-4"><h3 class="px-3 py-2 text-xs text-fg/50 flex gap-2 items-center"><Pin class="w-3 h-3" />{{ t('library.pinned') }}</h3><LibraryRow v-for="item in view.pinned" :key="item.id" :item="item" /></section>
      <LibraryGroup v-for="group in view.groups.filter(g=>!query.trim() || g.matching)" :key="group.id" :group="group" :searching="!!query.trim()" />
      <p v-if="!matching && !library.loading" class="text-sm text-fg/45 text-center py-7 px-3">{{ query.trim() ? t('library.noResults') : t('library.empty') }}</p>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue';
import { FolderPlus, Pin, Search } from '@lucide/vue';
import { useLibraryStore } from '@/stores/libraryStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
import { visibleLibrary } from '@/utils/libraryView';
import type { LibrarySort } from '@/types/library';
import LibraryGroup from './LibraryGroup.vue';
import LibraryRow from './LibraryRow.vue';
import LibraryNameEditor from './LibraryNameEditor.vue';
const library=useLibraryStore(), settings=useSettingsStore(), {t}=useI18n();
const query=ref(''), creating=ref(false);
const view=computed(()=>visibleLibrary(library.snapshot,query.value,settings.history.recent_files));
const matching=computed(()=>view.value.groups.reduce((n,g)=>n+g.matching,0));
function startCreate(event:MouseEvent) {(event.currentTarget as HTMLButtonElement).focus();creating.value=true;}
function create(name:string) {return library.apply({type:'create_project',id:crypto.randomUUID(),name});}
function rename(name:string) {const target=library.renameTarget;return target ? library.apply({type:'rename_recording',id:target.id,name}) : Promise.resolve(false);}
async function sort(event:Event) {const select=event.target as HTMLSelectElement;if(!await library.setSort(select.value as LibrarySort))select.value=library.snapshot.preferences.sort;}
</script>
