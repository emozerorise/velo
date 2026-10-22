<template>
  <aside
    ref="panel"
    class="reading-sidebar bg-chrome border-r border-fg/10 flex flex-col shrink-0"
    :class="{ 'is-open': open }"
    :aria-label="t('library.title')"
    :role="modal ? 'dialog' : undefined"
    :aria-modal="modal ? true : undefined"
    @keydown="panelKey"
  >
    <div
      data-tauri-drag-region
      class="h-[88px] shrink-0 flex items-center gap-3 px-5 pt-5"
    >
      <span class="sidebar-brand-mark flex items-center justify-center w-8 h-8 rounded-[10px] bg-blue-600 text-white shrink-0" aria-hidden="true">
        <AudioLines class="w-[22px] h-[22px]" :stroke-width="2" />
      </span>
      <span class="sidebar-wordmark text-[26px] leading-none font-semibold tracking-[-0.045em] text-fg">Velo</span>
      <button
        class="sidebar-close ml-auto p-2 rounded-lg text-fg/60 hover:bg-fg/5 focus-visible:ring-2 focus-visible:ring-accent"
        :aria-label="t('workspace.closePlaylist')"
        @click="emit('close')"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
    <div class="flex gap-2 px-4 pt-2 pb-5">
      <button
        class="sidebar-open-file min-w-0 flex-1 flex items-center gap-3 rounded-[10px] px-4 h-11 text-sm font-medium text-white transition-colors focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-chrome"
        :title="t('workspace.openFile')"
        @click="openFileDialog"
      >
        <Plus class="w-[18px] h-[18px] shrink-0" aria-hidden="true" />{{
          t('workspace.openFile')
        }}
      </button>
      <button
        class="w-11 h-11 shrink-0 flex items-center justify-center bg-fg/5 text-fg/60 hover:text-fg hover:bg-fg/10 rounded-[10px] transition-colors focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-chrome"
        :aria-label="t('app.openFolder')"
        :title="t('app.openFolder')"
        @click="openDirectoryDialog"
      >
        <FolderOpen class="w-[18px] h-[18px]" aria-hidden="true" />
      </button>
    </div>
    <LibraryBrowser />
    <button
      class="flex items-center gap-3 px-5 py-5 text-base text-fg/65 hover:text-fg border-t border-fg/5"
      @click="settings.openSettings()"
    >
      <Settings class="w-4 h-4" aria-hidden="true" />{{ t('topbar.settings') }}
    </button>
  </aside>
</template>

<script setup lang="ts">
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import {
  AudioLines,
  Plus,
  FolderOpen,
  Settings,
  X,
} from '@lucide/vue';
import LibraryBrowser from '@/components/library/LibraryBrowser.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useI18n } from '@/composables/useI18n';

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ (event: 'close'): void }>();
const settings = useSettingsStore();
const { openFileDialog, openDirectoryDialog } = useKeyboardShortcuts();
const { t } = useI18n();
const workspace = useWorkspaceStore();
const panel = ref<HTMLElement>();
const modal = computed(() => props.open && workspace.narrow);
let previousFocus: HTMLElement | null = null;
function restoreFocus() {
  if (previousFocus?.isConnected) previousFocus.focus();
  previousFocus = null;
}
function controls() {
  return Array.from(panel.value?.querySelectorAll<HTMLElement>('button, input, select, textarea, [href], [tabindex]') ?? [])
    .filter(el => el.tabIndex >= 0 && !el.hasAttribute('disabled') && el.getClientRects().length > 0);
}
function panelKey(event: KeyboardEvent) {
  if (!modal.value || event.isComposing) return;
  if (event.key === 'Escape') {
    event.preventDefault();event.stopPropagation();emit('close');
  }
  if (event.key !== 'Tab') return;
  event.preventDefault();event.stopPropagation();
  const items=controls();if (!items.length) return;
  const index=items.indexOf(document.activeElement as HTMLElement);
  items[event.shiftKey ? (index <= 0 ? items.length-1 : index-1) : (index+1)%items.length].focus();
}
watch(modal, async open => {
  if (!open) { restoreFocus();return; }
  previousFocus=document.activeElement as HTMLElement | null;
  await nextTick();if(modal.value) controls()[0]?.focus();
}, {flush:'post'});
onScopeDispose(restoreFocus);
</script>

<style scoped>
/* Keep the brand stable when the user changes the reading font. */
.sidebar-wordmark {
  font-family: 'Prompt', sans-serif;
}
.sidebar-open-file {
  background: #102f70;
}
.sidebar-open-file:hover {
  background: #19428f;
}
:global(.dark) .sidebar-open-file {
  background: #254c94;
}
:global(.dark) .sidebar-open-file:hover {
  background: #305ca9;
}
.reading-sidebar {
  width: 280px;
}
.sidebar-close {
  display: none;
}
@media (max-width: 899px) {
  .reading-sidebar {
    display: none;
  }
  .reading-sidebar.is-open {
    display: flex;
    position: absolute;
    inset: 0 auto 0 0;
    z-index: 5;
    box-shadow: 16px 0 48px rgb(0 0 0 / 0.35);
  }
  .sidebar-close {
    display: block;
  }
}
</style>
