import { onMounted, onUnmounted, ref, type Ref } from 'vue';
import { useControlsHold } from './useAutoHideControls';

/**
 * Popover open-state that closes on an outside click or Escape.
 * Attach `rootRef` to the element wrapping both the trigger and the panel.
 */
export function useDismissable() {
  const isOpen = ref(false);
  const rootRef: Ref<HTMLElement | null> = ref(null);

  // An open menu must not be swept away by the controls auto-hide.
  useControlsHold(isOpen);

  function close() {
    isOpen.value = false;
  }

  function toggle() {
    isOpen.value = !isOpen.value;
  }

  function handlePointerDown(e: MouseEvent) {
    if (!isOpen.value || !rootRef.value) return;
    if (!rootRef.value.contains(e.target as Node)) {
      close();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (isOpen.value && e.key === 'Escape') {
      e.stopPropagation();
      close();
    }
  }

  onMounted(() => {
    window.addEventListener('mousedown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener('mousedown', handlePointerDown);
    window.removeEventListener('keydown', handleKeyDown);
  });

  return { isOpen, rootRef, close, toggle };
}
