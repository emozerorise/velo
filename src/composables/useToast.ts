import { ref } from 'vue';

export interface ToastMessage {
  id: string;
  text: string;
}

const toasts = ref<ToastMessage[]>([]);

export function useToast() {
  function showToast(text: string, durationMs = 1500) {
    const id = crypto.randomUUID();
    toasts.value.push({ id, text });

    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, durationMs);
  }

  return {
    toasts,
    showToast,
  };
}
