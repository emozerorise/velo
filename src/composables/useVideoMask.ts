import { computed, onMounted, onScopeDispose, shallowReactive, watch } from 'vue';

const masks = shallowReactive(new Set<symbol>());
export const isVideoMasked = computed(() => masks.size > 0);

export function useVideoMask(open: () => boolean): void {
  const token = Symbol('video-mask');
  onMounted(() => watch(open, (active) => {
    if (active) masks.add(token);
    else masks.delete(token);
  }, { immediate: true, flush: 'sync' }));
  onScopeDispose(() => masks.delete(token));
}
