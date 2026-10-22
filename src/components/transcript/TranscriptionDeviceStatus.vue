<template>
  <p
    v-if="store.execution"
    role="status"
    class="shrink-0 px-6 py-2 text-xs border-b border-fg/10"
    :class="store.execution.fallback ? 'text-amber-500' : 'text-fg/55'"
  >
    {{ label }}
  </p>
</template>
<script setup lang="ts">
import { computed } from "vue";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useI18n } from "@/composables/useI18n";
const store = useTranscriptStore();
const { t } = useI18n();
const label = computed(() => {
  if (store.execution?.fallback === "no_gpu")
    return t("transcript.device.noGpu");
  if (store.execution?.fallback === "gpu_failed")
    return t("transcript.device.gpuFailed");
  return t("transcript.device.used", {
    device: store.execution?.device.toUpperCase() ?? "",
  });
});
</script>
