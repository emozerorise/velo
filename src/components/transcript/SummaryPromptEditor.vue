<template>
  <BaseModal :is-open="open" :title="t('summary.promptTitle')" @close="close">
    <form class="space-y-4" @submit.prevent="save">
      <div>
        <label
          for="summary-format"
          class="block text-sm font-medium text-fg/90 mb-2"
          >{{ t('summary.formatLabel') }}</label
        >
        <select
          id="summary-format"
          :value="format"
          :autofocus="initialDraft === undefined"
          :disabled="settings.summaryOptionsSaving"
          class="w-full rounded-xl border border-fg/15 bg-surface px-3 py-2.5 text-sm text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
          aria-describedby="summary-format-help"
          @change="changeFormat"
        >
          <option value="meeting">{{ t('summary.format.meeting') }}</option>
          <option value="key_points">
            {{ t('summary.format.key_points') }}
          </option>
          <option value="custom">{{ t('summary.format.custom') }}</option>
        </select>
      </div>
      <p class="text-sm leading-relaxed text-fg/65">
        {{ t('summary.promptHelp') }}
      </p>
      <p
        id="summary-format-help"
        class="rounded-xl bg-accent/[0.06] px-3 py-2 text-sm leading-relaxed text-fg/75"
      >
        {{ t(`summary.formatHelp.${format}`) }}
      </p>
      <div>
        <p class="text-xs text-fg/50 mb-2">{{ t('summary.promptExamples') }}</p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="example in examples"
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
        <label
          for="summary-custom-prompt"
          class="block text-sm font-medium text-fg/90 mb-2"
          >{{ t('summary.promptField') }}</label
        >
        <textarea
          id="summary-custom-prompt"
          v-model="draft"
          rows="6"
          :autofocus="initialDraft !== undefined"
          :disabled="settings.summaryOptionsSaving"
          :placeholder="t('settings.aiInstructionsPlaceholder')"
          class="w-full min-h-32 resize-y rounded-xl border border-fg/15 bg-inset/20 px-3 py-3 text-sm leading-relaxed text-fg/90 placeholder:text-fg/30 focus:outline focus:outline-2 focus:outline-accent"
          @keydown.esc.stop.prevent="close"
        />
      </div>
      <p class="text-xs leading-relaxed text-fg/50">
        {{ t('summary.promptScope') }}
      </p>
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
          {{
            settings.summaryOptionsSaving
              ? t('summary.promptSaving')
              : t('summary.promptSave')
          }}
        </button>
      </div>
    </form>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import BaseModal from '@/components/common/BaseModal.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
import type { SummaryFormat } from '@/types/settings';

const props = defineProps<{ open: boolean; initialDraft?: string }>();
const emit = defineEmits<{ (event: 'close'): void }>();
const settings = useSettingsStore();
const { t } = useI18n();
const draft = ref('');
const format = ref<SummaryFormat>('meeting');
const error = ref('');
const MEETING_EXAMPLES = [
  { label: 'summary.promptExecutive', text: 'summary.promptExecutiveText' },
  { label: 'summary.promptActions', text: 'summary.promptActionsText' },
  { label: 'summary.promptDevelopment', text: 'summary.promptDevelopmentText' },
] as const;
const FLEXIBLE_EXAMPLES = [
  { label: 'summary.promptParagraph', text: 'summary.promptParagraphText' },
  { label: 'summary.promptKeyIdeas', text: 'summary.promptKeyIdeasText' },
  {
    label: 'summary.promptPerspectives',
    text: 'summary.promptPerspectivesText',
  },
] as const;
const KEY_POINT_EXAMPLES = [
  { label: 'summary.promptConcise', text: 'summary.promptConciseText' },
  { label: 'summary.promptExplain', text: 'summary.promptExplainText' },
  { label: 'summary.promptEvidence', text: 'summary.promptEvidenceText' },
] as const;
const examples = computed(
  () =>
    ({
      meeting: MEETING_EXAMPLES,
      key_points: KEY_POINT_EXAMPLES,
      custom: FLEXIBLE_EXAMPLES,
    })[format.value],
);

function changeFormat(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  if (value === 'meeting' || value === 'key_points' || value === 'custom')
    format.value = value;
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      format.value = settings.settings.summary.format ?? 'meeting';
      draft.value =
        props.initialDraft ?? settings.settings.summary.instructions;
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
  if (
    await settings.saveSummaryOptions({
      format: format.value,
      instructions: draft.value,
    })
  )
    emit('close');
  else error.value = t('summary.optionsSaveError');
}
</script>
