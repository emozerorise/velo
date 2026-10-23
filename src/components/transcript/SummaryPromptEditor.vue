<template>
  <BaseModal
    :is-open="open"
    :title="t('summary.promptTitle')"
    wide
    @close="close"
  >
    <form class="flex max-h-[calc(70vh-2rem)] flex-col" @submit.prevent="save">
      <div class="min-h-0 space-y-5 overflow-y-auto pb-4 pr-1">
        <fieldset :disabled="settings.summaryOptionsSaving">
          <legend class="mb-3 text-sm font-semibold">
            {{ t("summary.formatLabel") }}
          </legend>
          <div class="grid gap-2 sm:grid-cols-3">
            <label
              v-for="choice in formats"
              :key="choice"
              class="cursor-pointer rounded-xl border p-3 transition-colors"
              :class="
                format === choice
                  ? 'border-accent bg-accent/10'
                  : 'border-fg/15 hover:bg-fg/5'
              "
            >
              <input
                type="radio"
                name="summary-format"
                :value="choice"
                :checked="format === choice"
                :autofocus="!focusDraft && format === choice"
                :disabled="settings.summaryOptionsSaving"
                class="mr-2 accent-blue-600"
                @change="format = choice"
              />
              <span class="text-sm font-medium">{{
                t(`summary.format.${choice}`)
              }}</span>
            </label>
          </div>
        </fieldset>
        <fieldset
          v-if="format === 'meeting'"
          :disabled="settings.summaryOptionsSaving"
        >
          <legend class="mb-2 text-sm font-semibold">
            {{ t("summary.detailLabel") }}
          </legend>
          <div class="grid gap-2 sm:grid-cols-2">
            <label
              v-for="choice in details"
              :key="choice"
              class="cursor-pointer rounded-xl border p-3"
              :class="
                detail === choice
                  ? 'border-accent bg-accent/10'
                  : 'border-fg/15'
              "
            >
              <input
                type="radio"
                name="meeting-detail"
                :value="choice"
                :checked="detail === choice"
                :data-testid="`summary-detail-${choice}`"
                :disabled="settings.summaryOptionsSaving"
                class="mr-2 accent-blue-600"
                @change="detail = choice"
              />
              <span class="text-sm font-medium">{{
                t(`summary.detail.${choice}`)
              }}</span>
              <p class="mt-1 text-xs leading-relaxed text-fg/65">
                {{ t(`summary.detailHelp.${choice}`) }}
              </p>
            </label>
          </div>
        </fieldset>
        <section
          class="rounded-xl border border-accent/15 bg-accent/[0.04] p-4"
          aria-labelledby="summary-expected-title"
        >
          <h4 id="summary-expected-title" class="mb-2 text-sm font-semibold">
            {{ t("summary.expectedOutput") }}
          </h4>
          <p class="text-sm leading-relaxed text-fg/70">
            {{ t(`summary.formatHelp.${format}`) }}
          </p>
          <details
            v-if="format === 'meeting'"
            class="mt-3 text-xs leading-relaxed text-fg/65"
          >
            <summary tabindex="0" class="cursor-pointer">
              {{ t("summary.previewInstructions") }}
            </summary>
            <p class="mt-2">
              {{
                t(
                  detail === "detailed"
                    ? "summary.detailPrompt"
                    : "summary.detailHelp.standard",
                )
              }}
            </p>
          </details>
        </section>
        <div>
          <button
            type="button"
            data-testid="summary-advanced-toggle"
            :aria-expanded="advancedOpen"
            aria-controls="summary-advanced-fields"
            :disabled="settings.summaryOptionsSaving"
            class="rounded-lg px-1 py-2 text-sm font-medium text-accent focus-visible:outline focus-visible:outline-accent"
            @click="advancedOpen = !advancedOpen"
          >
            {{ t("summary.advancedOptions") }} {{ advancedOpen ? "−" : "+" }}
          </button>
          <p v-if="draft.trim() && !advancedOpen" class="text-xs text-fg/60">
            {{ t("summary.savedInstructions") }}
          </p>
          <div
            v-show="advancedOpen"
            id="summary-advanced-fields"
            data-testid="summary-advanced-fields"
            class="space-y-3 pt-2"
          >
            <p class="text-xs text-fg/60">{{ t("summary.promptExamples") }}</p>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="example in examples"
                :key="example.label"
                type="button"
                :disabled="settings.summaryOptionsSaving"
                class="rounded-lg border border-accent/20 bg-accent/[0.07] px-3 py-2 text-xs text-accent hover:bg-accent/15 focus-visible:outline focus-visible:outline-accent disabled:opacity-40"
                @click="applyExample(t(example.text))"
              >
                {{ t(example.label) }}
              </button>
            </div>
            <div
              v-if="pendingExample !== null"
              role="group"
              :aria-label="t('summary.replacePromptQuestion')"
              class="rounded-xl border border-accent/25 p-3"
            >
              <p class="mb-2 text-sm">
                {{ t("summary.replacePromptQuestion") }}
              </p>
              <button
                type="button"
                data-testid="summary-example-keep"
                :disabled="settings.summaryOptionsSaving"
                class="rounded-lg border border-fg/15 px-3 py-2 text-xs"
                @click="pendingExample = null"
              >
                {{ t("summary.keepPrompt") }}
              </button>
              <button
                type="button"
                data-testid="summary-example-replace"
                :disabled="settings.summaryOptionsSaving"
                class="ml-2 rounded-lg bg-accent px-3 py-2 text-xs text-white"
                @click="replaceExample"
              >
                {{ t("summary.replacePrompt") }}
              </button>
            </div>
            <label
              for="summary-custom-prompt"
              class="block text-sm font-medium"
              >{{ t("summary.promptField") }}</label
            >
            <textarea
              id="summary-custom-prompt"
              v-model="draft"
              rows="5"
              :autofocus="focusDraft"
              :disabled="settings.summaryOptionsSaving"
              :placeholder="t('settings.aiInstructionsPlaceholder')"
              class="w-full min-h-32 resize-y rounded-xl border border-fg/15 bg-inset/20 px-3 py-3 text-sm leading-relaxed text-fg/90 placeholder:text-fg/30 focus:outline focus:outline-2 focus:outline-accent"
              @keydown.esc.stop.prevent="close"
            />
            <button
              type="button"
              class="text-xs text-fg/60 hover:text-fg"
              :disabled="settings.summaryOptionsSaving"
              @click="
                draft = '';
                pendingExample = null;
              "
            >
              {{ t("summary.promptReset") }}
            </button>
          </div>
        </div>
        <p class="text-xs leading-relaxed text-fg/50">
          {{ t("summary.promptScope") }}
        </p>
      </div>
      <footer class="shrink-0 border-t border-fg/10 pt-3">
        <p v-if="error" role="alert" class="mb-2 text-xs text-danger">
          {{ error }}
        </p>
        <div class="flex justify-end gap-2">
          <button
            type="button"
            class="rounded-lg border border-fg/15 px-3 py-2 text-sm disabled:opacity-40"
            :disabled="settings.summaryOptionsSaving"
            @click="close"
          >
            {{ t("summary.promptCancel") }}
          </button>
          <button
            type="submit"
            class="rounded-lg bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-40"
            :disabled="settings.summaryOptionsSaving"
          >
            {{
              settings.summaryOptionsSaving
                ? t("summary.promptSaving")
                : t("summary.promptSave")
            }}
          </button>
        </div>
      </footer>
    </form>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import BaseModal from "@/components/common/BaseModal.vue";
import { useSettingsStore } from "@/stores/settingsStore";
import { useI18n } from "@/composables/useI18n";
import type { MeetingDetail, SummaryFormat } from "@/types/settings";

const props = defineProps<{ open: boolean; initialDraft?: string }>();
const emit = defineEmits<{ (event: "close"): void }>();
const settings = useSettingsStore();
const { t } = useI18n();
const draft = ref("");
const format = ref<SummaryFormat>("meeting");
const detail = ref<MeetingDetail>("standard");
const advancedOpen = ref(false);
const focusDraft = ref(false);
const pendingExample = ref<string | null>(null);
const error = ref("");
const formats = ["meeting", "key_points", "custom"] as const;
const details = ["standard", "detailed"] as const;
const MEETING_EXAMPLES = [
  { label: "summary.promptExecutive", text: "summary.promptExecutiveText" },
  { label: "summary.promptActions", text: "summary.promptActionsText" },
  { label: "summary.promptDevelopment", text: "summary.promptDevelopmentText" },
] as const;
const FLEXIBLE_EXAMPLES = [
  { label: "summary.promptParagraph", text: "summary.promptParagraphText" },
  { label: "summary.promptKeyIdeas", text: "summary.promptKeyIdeasText" },
  {
    label: "summary.promptPerspectives",
    text: "summary.promptPerspectivesText",
  },
] as const;
const KEY_POINT_EXAMPLES = [
  { label: "summary.promptConcise", text: "summary.promptConciseText" },
  { label: "summary.promptExplain", text: "summary.promptExplainText" },
  { label: "summary.promptEvidence", text: "summary.promptEvidenceText" },
] as const;
const examples = computed(
  () =>
    ({
      meeting: MEETING_EXAMPLES,
      key_points: KEY_POINT_EXAMPLES,
      custom: FLEXIBLE_EXAMPLES,
    })[format.value],
);
watch(format, (value) => {
  pendingExample.value = null;
  if (value === "custom") advancedOpen.value = true;
});
watch(
  () => props.open,
  (open) => {
    if (!open) return;
    format.value = settings.settings.summary.format ?? "meeting";
    detail.value = settings.settings.summary.meeting_detail ?? "standard";
    draft.value = props.initialDraft ?? settings.settings.summary.instructions;
    focusDraft.value =
      props.initialDraft !== undefined || format.value === "custom";
    advancedOpen.value = focusDraft.value;
    pendingExample.value = null;
    error.value = "";
  },
  { immediate: true },
);
function applyExample(text: string) {
  if (draft.value.trim()) pendingExample.value = text;
  else draft.value = text;
}
function replaceExample() {
  if (pendingExample.value !== null) draft.value = pendingExample.value;
  pendingExample.value = null;
}
function close() {
  if (!settings.summaryOptionsSaving) emit("close");
}
async function save() {
  error.value = "";
  if (
    await settings.saveSummaryOptions({
      format: format.value,
      meeting_detail: detail.value,
      instructions: draft.value,
    })
  )
    emit("close");
  else error.value = t("summary.optionsSaveError");
}
</script>
