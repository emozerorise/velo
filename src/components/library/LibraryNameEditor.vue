<template>
  <form data-testid="name-form" class="space-y-2 p-2 rounded-xl bg-fg/5" @submit.prevent="submit" @keydown.esc.stop.prevent="emit('cancel')">
    <label class="block text-xs text-fg/65">{{ label }}
      <input ref="input" v-model="draft" data-testid="name-input" class="block mt-1 w-full rounded-lg border border-fg/20 bg-chrome px-2 py-2 text-sm text-fg" :disabled="busy" />
    </label>
    <p v-if="invalid" role="alert" class="text-xs text-danger">{{ t('library.nameInvalid') }}</p>
    <div class="flex gap-2 justify-end">
      <button type="button" data-testid="name-cancel" class="px-2 py-1 text-xs text-fg/65" :disabled="busy" @click="emit('cancel')">{{ t('library.cancel') }}</button>
      <button type="submit" class="px-3 py-1 rounded-lg bg-accent text-white text-xs" :disabled="busy">{{ busy ? t('library.saving') : t('library.save') }}</button>
    </div>
  </form>
</template>
<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from '@/composables/useI18n';
const props = defineProps<{initial: string; label: string; save: (name: string) => Promise<boolean>}>();
const emit = defineEmits<{cancel: []; saved: []}>();
const { t } = useI18n();
const draft = ref(props.initial), busy = ref(false), invalid = ref(false);
const input = ref<HTMLInputElement>();
const returnTarget = typeof document === 'undefined' ? null : document.activeElement as HTMLElement | null;
onMounted(() => input.value?.focus());
onUnmounted(() => { if (returnTarget?.isConnected) returnTarget.focus(); });
async function submit() {
  const value = draft.value.trim();
  invalid.value = !value || Array.from(value).length > 120;
  if (invalid.value || busy.value) return;
  busy.value = true;
  try { if (await props.save(value)) emit('saved'); } finally { busy.value = false; }
}
</script>
