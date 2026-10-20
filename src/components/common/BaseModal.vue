<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70"
        @click.self="$emit('close')"
      >
        <div
          ref="dialog"
          role="dialog"
          aria-modal="true"
          :aria-label="title"
          tabindex="-1"
          class="relative w-full max-w-lg overflow-hidden bg-surface border border-fg/10 rounded-2xl shadow-[0_24px_60px_-16px_rgba(0,0,0,0.95)] text-fg animate-in fade-in zoom-in-95 duration-150"
          @keydown.stop="modalKey"
        >
          <!-- Header -->
          <div
            class="flex items-center justify-between px-6 py-4 border-b border-fg/10"
          >
            <h3 class="text-[15px] font-semibold text-fg">
              {{ title }}
            </h3>
            <button
              :aria-label="t('common.close')"
              class="p-1 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
              @click="$emit('close')"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="px-6 py-4 max-h-[70vh] overflow-y-auto">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onScopeDispose, ref, watch } from 'vue';
import { useVideoMask } from '@/composables/useVideoMask';
import { useI18n } from '@/composables/useI18n';
const props = defineProps<{
  isOpen: boolean;
  title: string;
}>();
useVideoMask(() => props.isOpen);
const { t } = useI18n();
const dialog = ref<HTMLElement | null>(null);
let previousFocus: HTMLElement | null = null;

const emit = defineEmits<{
  (e: 'close'): void;
}>();
function focusables(): HTMLElement[] {
  return Array.from(
    dialog.value?.querySelectorAll<HTMLElement>(
      'button, input, select, textarea, [href], [tabindex]',
    ) ?? [],
  ).filter(
    (element) =>
      element.tabIndex >= 0 &&
      !element.hasAttribute('disabled') &&
      element.getClientRects().length > 0,
  );
}
function restoreFocus() {
  if (previousFocus?.isConnected) previousFocus.focus();
  previousFocus = null;
}
function modalKey(event: KeyboardEvent) {
  if (event.isComposing) return;
  if (event.key === 'Escape') {
    event.preventDefault();
    emit('close');
  }
  if (event.key !== 'Tab') return;
  event.preventDefault();
  const elements = focusables();
  if (!elements.length) {
    dialog.value?.focus();
    return;
  }
  const index = elements.indexOf(document.activeElement as HTMLElement);
  const next = event.shiftKey
    ? index <= 0
      ? elements.length - 1
      : index - 1
    : (index + 1) % elements.length;
  elements[next].focus();
}
onMounted(() =>
  watch(
    () => props.isOpen,
    async (open) => {
      if (!open) {
        restoreFocus();
        return;
      }
      previousFocus =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      await nextTick();
      if (!props.isOpen) return;
      const target =
        dialog.value?.querySelector<HTMLElement>('[autofocus]') ??
        focusables()[0] ??
        dialog.value;
      target?.focus();
    },
    { immediate: true, flush: 'post' },
  ),
);
onScopeDispose(restoreFocus);
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
