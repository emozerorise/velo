<template>
  <div class="min-w-0 flex-1 space-y-2">
    <select
      data-testid="model-select"
      :aria-label="t('settings.aiModel')"
      :value="custom ? 'custom' : model ? `model:${model}` : ''"
      :disabled="disabled"
      class="w-full min-w-0 bg-inset border border-fg/15 rounded-lg px-3 py-2 text-sm text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50"
      @change="selectModel"
    >
      <option v-if="!model" value="" disabled>{{ t('settings.aiModelChoose') }}</option>
      <option v-if="model && !choices.includes(model)" :value="`model:${model}`">
        {{ model }} — {{ t('settings.aiModelSaved') }}
      </option>
      <option v-for="name in choices" :key="name" :value="`model:${name}`">{{ name }}</option>
      <option value="custom">{{ t('settings.aiModelCustom') }}</option>
    </select>
    <input
      v-if="custom"
      ref="customInput"
      data-testid="model-custom"
      :value="draft"
      :aria-label="t('settings.aiModelCustomName')"
      :aria-invalid="invalid"
      :disabled="disabled"
      :placeholder="t('settings.aiModelCustomName')"
      type="text"
      spellcheck="false"
      class="w-full min-w-0 bg-inset/40 border border-fg/15 rounded-lg px-3 py-2 text-sm text-fg font-mono focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50"
      @change="saveCustom"
    />
    <p v-if="invalid" role="alert" class="text-xs text-danger">{{ t('settings.aiModelRequired') }}</p>
    <p v-else-if="!choices.length" class="text-xs text-fg/50">{{ t('settings.aiModelEmpty') }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { useI18n } from '@/composables/useI18n';

const props = defineProps<{ model: string; models: string[]; disabled?: boolean }>();
const emit = defineEmits<{ change: [model: string] }>();
const { t } = useI18n();
const custom = ref(false);
const draft = ref('');
const invalid = ref(false);
const customInput = ref<HTMLInputElement | null>(null);
const choices = computed(() => [...new Set(props.models.filter(name => name.trim()))]);

async function selectModel(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  invalid.value = false;
  custom.value = value === 'custom';
  if (custom.value) {
    draft.value = props.model;
    await nextTick();
    customInput.value?.focus();
  } else if (value.startsWith('model:')) {
    emit('change', value.slice(6));
  }
}

function saveCustom(event: Event) {
  draft.value = (event.target as HTMLInputElement).value.trim();
  invalid.value = !draft.value;
  if (!invalid.value) emit('change', draft.value);
}
</script>
