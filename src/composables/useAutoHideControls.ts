import { ref, onMounted, onUnmounted, watch, type Ref } from 'vue';
import { usePlayerStore } from '@/stores/playerStore';

/**
 * Number of UI elements that currently need the controls to stay put --
 * an open menu, an in-progress scrub. Hiding the controls unmounts them,
 * which would tear down whatever the user is interacting with.
 */
const holdCount = ref(0);

/** Keeps the controls visible for as long as `active` is true. */
export function useControlsHold(active: Ref<boolean>) {
  let held = false;

  function release() {
    if (held) {
      holdCount.value--;
      held = false;
    }
  }

  const stop = watch(
    active,
    (isActive) => {
      if (isActive && !held) {
        holdCount.value++;
        held = true;
      } else if (!isActive) {
        release();
      }
    },
    { immediate: true }
  );

  onUnmounted(() => {
    stop();
    release();
  });
}

export function useAutoHideControls(timeoutMs = 2500) {
  const player = usePlayerStore();
  const areControlsVisible = ref(true);
  let timer: number | null = null;

  function showControls() {
    areControlsVisible.value = true;
    resetTimer();
  }

  function resetTimer() {
    if (timer !== null) {
      window.clearTimeout(timer);
      timer = null;
    }

    if (player.state === 'playing') {
      timer = window.setTimeout(() => {
        if (holdCount.value > 0 || player.isDraggingTimeline) {
          resetTimer();
          return;
        }
        areControlsVisible.value = false;
      }, timeoutMs);
    }
  }

  function handleMouseMove() {
    showControls();
  }

  watch(
    () => player.state,
    (newState) => {
      if (newState === 'playing') {
        resetTimer();
      } else {
        // Always show controls when paused, loading, or idle
        areControlsVisible.value = true;
        if (timer !== null) {
          window.clearTimeout(timer);
          timer = null;
        }
      }
    }
  );

  onMounted(() => {
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mousedown', handleMouseMove);
    resetTimer();
  });

  onUnmounted(() => {
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mousedown', handleMouseMove);
    if (timer !== null) {
      window.clearTimeout(timer);
    }
  });

  return {
    areControlsVisible,
    showControls,
  };
}
