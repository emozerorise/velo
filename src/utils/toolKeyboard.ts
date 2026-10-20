import { isVideoMasked } from '@/composables/useVideoMask';

export function toolMayClose(event: KeyboardEvent): boolean {
  const target = event.target;
  return (
    !isVideoMasked.value &&
    !(
      target instanceof HTMLElement &&
      (['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName) ||
        target.isContentEditable)
    )
  );
}
