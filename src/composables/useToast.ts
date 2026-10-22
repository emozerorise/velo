import { ref } from 'vue';

export interface ToastMessage {
  id: string;
  text: string;
  key?: string;
}

const toasts = ref<ToastMessage[]>([]);
const timers = new Map<string, ReturnType<typeof setTimeout>>();

export function useToast() {
  function showToast(text: string, durationMs = 1500, key?: string) {
    const existing = key ? toasts.value.find((toast) => toast.key === key) : undefined;
    const id = existing?.id ?? crypto.randomUUID();
    if (existing) {
      existing.text = text;
      clearTimeout(timers.get(id));
    } else {
      toasts.value.push({ id, text, key });
    }
    timers.set(
      id,
      setTimeout(() => {
        toasts.value = toasts.value.filter((t) => t.id !== id);
        timers.delete(id);
      }, durationMs),
    );
  }

  return {
    toasts,
    showToast,
  };
}
