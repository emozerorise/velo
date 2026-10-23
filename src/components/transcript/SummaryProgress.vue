<template>
  <div class="summary-progress" data-testid="summary-progress">
    <div class="progress-card">
      <div class="progress-heading">
        <div class="progress-icon"><Sparkles aria-hidden="true" /></div>
        <div>
          <p class="progress-eyebrow">{{ formatLabel }}</p>
          <h2>{{ t("summary.progressTitle") }}</h2>
        </div>
        <span class="progress-live"
          ><span />{{ t("summary.progressWorking") }}</span
        >
      </div>
      <p class="progress-description">{{ t("summary.progressDescription") }}</p>
      <div class="progress-status" role="status" aria-live="polite">
        <Loader2 class="w-4 h-4 motion-safe:animate-spin" aria-hidden="true" />
        <span>{{ label }}</span>
        <span v-if="progress >= 0" class="ml-auto font-mono"
          >{{ percent }}%</span
        >
      </div>
      <div
        class="progress-track"
        role="progressbar"
        :aria-label="label"
        :aria-valuenow="progress >= 0 ? percent : undefined"
        :aria-valuemin="0"
        :aria-valuemax="100"
      >
        <div
          :class="{ indeterminate: progress < 0 }"
          :style="progress >= 0 ? { width: `${percent}%` } : undefined"
        />
      </div>
      <ol class="progress-steps" :aria-label="t('summary.progressSteps')">
        <li
          v-for="(step, index) in steps"
          :key="step"
          :class="{ active: index === activeStep, done: index < activeStep }"
          :aria-current="index === activeStep ? 'step' : undefined"
        >
          <span class="step-dot"
            ><Check v-if="index < activeStep" aria-hidden="true" /><span
              v-else
              >{{ index + 1 }}</span
            ></span
          >
          <span>{{ t(step) }}</span>
        </li>
      </ol>
      <div class="progress-footer">
        <span>{{ t("summary.progressKeep") }}</span>
        <button type="button" @click="$emit('cancel')">
          {{ t("summary.cancel") }}
        </button>
      </div>
    </div>
    <p class="progress-note">{{ t("summary.runningNote") }}</p>
    <pre v-if="streamed" class="progress-stream">{{ streamed }}</pre>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Check, Loader2, Sparkles } from "@lucide/vue";
import { useI18n } from "@/composables/useI18n";
import type { ChainStage } from "@/types/summary";
const props = defineProps<{
  label: string;
  stage: ChainStage | null;
  progress: number;
  formatLabel: string;
  streamed: string;
}>();
defineEmits<{ (event: "cancel"): void }>();
const { t } = useI18n();
const legacySteps = [
  "summary.progressRead",
  "summary.progressVerify",
  "summary.progressArrange",
] as const;
const direct = computed(() => !['extracting_evidence', 'checking_evidence', 'composing_summary', 'reviewing'].includes(props.stage ?? ''));
const multipart = computed(() => ['writing_parts', 'checking_consistency', 'replacing_parts', 'assembling_summary'].includes(props.stage ?? ''));
const steps = computed(() => multipart.value
  ? ['summary.writingPartsLabel', 'summary.assemblingParts', 'summary.checkingReferences'] as const
  : direct.value ? ['summary.progressRead', 'summary.writingDirect', 'summary.checkingReferences'] as const : legacySteps);
const activeStep = computed(() => {
  if (props.stage === 'assembling_summary') return 1;
  if (props.stage === 'checking_consistency' || props.stage === 'replacing_parts') return 1;
  if (props.stage === 'checking_references') return 2;
  if (props.stage === 'writing_summary' || props.stage === 'reducing') return 1;
  if (props.stage === "composing_summary") return 2;
  if (props.stage === "checking_evidence" || props.stage === "reviewing")
    return 1;
  return 0;
});
const percent = computed(() =>
  Math.round(Math.min(1, Math.max(0, props.progress)) * 100),
);
</script>

<style scoped>
.summary-progress {
  width: 100%;
  max-width: 760px;
  margin: auto;
  padding: 48px 28px 24px;
}
.progress-card {
  padding: 32px;
  border: 1px solid rgb(var(--velo-accent) / 0.18);
  border-radius: 24px;
  background:
    linear-gradient(145deg, rgb(var(--velo-accent) / 0.055), transparent 65%),
    rgb(var(--velo-surface));
  box-shadow: 0 12px 40px rgb(var(--velo-accent) / 0.055);
}
.progress-heading {
  display: flex;
  align-items: center;
  gap: 16px;
}
.progress-icon {
  display: grid;
  place-items: center;
  flex-shrink: 0;
  width: 52px;
  height: 52px;
  border: 1px solid rgb(var(--velo-accent) / 0.16);
  border-radius: 16px;
  background: rgb(var(--velo-accent) / 0.09);
  color: rgb(var(--velo-accent));
}
.progress-icon svg {
  width: 25px;
  height: 25px;
}
.progress-eyebrow {
  font-size: 12px;
  color: rgb(var(--velo-fg) / 0.55);
  margin-bottom: 4px;
}
h2 {
  font-size: 23px;
  font-weight: 600;
  line-height: 1.5;
  color: rgb(var(--velo-fg) / 0.95);
}
.progress-live {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  white-space: nowrap;
  color: rgb(var(--velo-accent));
}
.progress-live > span {
  height: 6px;
  width: 6px;
  border-radius: 50%;
  background: currentColor;
}
.progress-description {
  margin: 20px 0 28px;
  font-size: 14px;
  line-height: 1.8;
  color: rgb(var(--velo-fg) / 0.65);
}
.progress-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: rgb(var(--velo-accent));
}
.progress-track {
  height: 5px;
  margin-top: 12px;
  overflow: hidden;
  border-radius: 99px;
  background: rgb(var(--velo-accent) / 0.1);
}
.progress-track > div {
  height: 100%;
  border-radius: inherit;
  background: rgb(var(--velo-accent));
  transition: width 0.25s ease;
}
.progress-track .indeterminate {
  width: 35%;
  animation: travel 2s ease-in-out infinite alternate;
}
.progress-steps {
  display: flex;
  gap: 16px;
  margin: 24px 0 30px;
}
.progress-steps li {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  line-height: 1.6;
  color: rgb(var(--velo-fg) / 0.45);
}
.step-dot {
  display: grid;
  place-items: center;
  flex-shrink: 0;
  width: 25px;
  height: 25px;
  border: 1px solid rgb(var(--velo-fg) / 0.14);
  border-radius: 50%;
  font-size: 11px;
}
.step-dot svg {
  width: 14px;
  height: 14px;
}
.progress-steps .active {
  color: rgb(var(--velo-accent));
  font-weight: 600;
}
.active .step-dot {
  background: rgb(var(--velo-accent));
  color: white;
  border-color: transparent;
  box-shadow: 0 0 0 4px rgb(var(--velo-accent) / 0.08);
}
.progress-steps .done {
  color: rgb(var(--velo-fg) / 0.65);
}
.done .step-dot {
  background: rgb(var(--velo-accent) / 0.08);
  color: rgb(var(--velo-accent));
  border-color: transparent;
}
.progress-footer {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-top: 20px;
  border-top: 1px solid rgb(var(--velo-fg) / 0.08);
  color: rgb(var(--velo-fg) / 0.55);
  font-size: 12px;
  line-height: 1.7;
}
.progress-footer button {
  margin-left: auto;
  flex-shrink: 0;
  padding: 8px 14px;
  border-radius: 10px;
  border: 1px solid rgb(var(--velo-fg) / 0.15);
  color: rgb(var(--velo-fg) / 0.75);
  background: rgb(var(--velo-surface));
}
.progress-footer button:hover {
  background: rgb(var(--velo-fg) / 0.05);
}
.progress-footer button:focus-visible {
  outline: 2px solid rgb(var(--velo-accent));
  outline-offset: 3px;
}
.progress-note {
  text-align: center;
  font-size: 12px;
  line-height: 1.8;
  color: rgb(var(--velo-fg) / 0.5);
  margin: 18px 12px 0;
}
.progress-stream {
  white-space: pre-wrap;
  margin-top: 24px;
  font: inherit;
  font-size: 14px;
  line-height: 1.8;
}
@keyframes travel {
  from {
    transform: translateX(-30%);
  }
  to {
    transform: translateX(210%);
  }
}
@media (prefers-reduced-motion: reduce) {
  .progress-track .indeterminate {
    animation: none;
    width: 100%;
    opacity: 0.45;
  }
}
@media (max-width: 700px), (max-height: 550px) {
  .summary-progress {
    padding: 20px 16px;
  }
  .progress-card {
    padding: 22px;
  }
  .progress-live {
    display: none;
  }
  h2 {
    font-size: 20px;
  }
  .progress-steps {
    gap: 10px;
  }
  .progress-steps li {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
