<template>
  <BaseModal :is-open="open" :title="t('summary.promptTitle')" @close="close">
    <form class="space-y-4" @submit.prevent="save" @keydown.stop>
      <p class="text-sm leading-relaxed text-fg/65">{{ t('summary.promptHelp') }}</p>
      <div>
        <p class="text-xs text-fg/50 mb-2">{{ t('summary.promptExamples') }}</p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="example in EXAMPLES"
            :key="example.label"
            type="button"
            class="rounded-lg border border-accent/20 bg-accent/[0.07] px-3 py-2 text-xs text-accent hover:bg-accent/15 focus-visible:outline focus-visible:outline-accent disabled:opacity-40"
            :disabled="settings.summaryOptionsSaving"
            @click="draft = t(example.text)"
          >
            {{ t(example.label) }}
          </button>
        </div>
      </div>
      <div>
        <label for="summary-custom-prompt" class="block text-sm font-medium text-fg/90 mb-2">{{
          t('summary.promptField')
        }}</label>
        <textarea
          id="summary-custom-prompt"
          v-model="draft"
          rows="6"
          autofocus
          :disabled="settings.summaryOptionsSaving"
          :placeholder="t('settings.aiInstructionsPlaceholder')"
          class="w-full min-h-32 resize-y rounded-xl border border-fg/15 bg-inset/20 px-3 py-3 text-sm leading-relaxed text-fg/90 placeholder:text-fg/30 focus:outline focus:outline-2 focus:outline-accent"
          @keydown.esc.stop.prevent="close"
        />
      </div>
      <p class="text-xs leading-relaxed text-fg/50">{{ t('summary.promptScope') }}</p>
      <p v-if="error" role="alert" class="text-xs text-danger">{{ error }}</p>
      <div class="flex flex-wrap items-center justify-end gap-2">
        <button
          type="button"
          class="mr-auto text-xs text-fg/60 hover:text-fg disabled:opacity-40"
          :disabled="settings.summaryOptionsSaving"
          @click="draft = ''"
        >
          {{ t('summary.promptReset') }}
        </button>
        <button
          type="button"
          class="rounded-lg border border-fg/10 px-3 py-2 text-sm text-fg/70 disabled:opacity-40"
          :disabled="settings.summaryOptionsSaving"
          @click="close"
        >
          {{ t('summary.promptCancel') }}
        </button>
        <button
          type="submit"
          class="rounded-lg bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-40"
          :disabled="settings.summaryOptionsSaving"
        >
          {{ settings.summaryOptionsSaving ? t('summary.promptSaving') : t('summary.promptSave') }}
        </button>
      </div>
    </form>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import BaseModal from '@/components/common/BaseModal.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ (event: 'close'): void }>();
const settings = useSettingsStore();
const { t } = useI18n();
const draft = ref('');
const error = ref('');
const EXAMPLES = [
  { label: 'summary.promptExecutive', text: 'summary.promptExecutiveText' },
  { label: 'summary.promptActions', text: 'summary.promptActionsText' },
  { label: 'summary.promptDevelopment', text: 'summary.promptDevelopmentText' },
] as const;

watch(
  () => props.open,
  (open) => {
    if (open) {
      draft.value = settings.settings.summary.instructions;
      error.value = '';
    }
  },
  { immediate: true },
);

function close() {
  if (!settings.summaryOptionsSaving) emit('close');
}
async function save() {
  error.value = '';
  if (await settings.saveSummaryOptions({ instructions: draft.value })) emit('close');
  else error.value = t('summary.optionsSaveError');
}
</script>
