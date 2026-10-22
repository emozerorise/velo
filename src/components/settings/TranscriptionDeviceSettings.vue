<template>
  <div class="space-y-2">
    <div class="flex items-center justify-between gap-4">
      <label for="transcription-device" class="font-medium text-fg/90">{{
        t("settings.transcriptionDevice")
      }}</label>
      <select
        id="transcription-device"
        data-testid="transcription-device"
        :value="store.settings.transcript.device"
        :disabled="store.transcriptDeviceSaving"
        aria-describedby="transcription-device-help"
        class="bg-inset/40 border border-fg/15 rounded-lg px-3 py-1.5 text-xs text-fg outline-none"
        @change="change"
      >
        <option value="auto">
          {{ t("settings.transcriptionDevice.auto") }}
        </option>
        <option value="gpu">GPU</option>
        <option value="cpu">CPU</option>
      </select>
    </div>
    <p
      id="transcription-device-help"
      class="text-xs text-fg/50 leading-relaxed"
    >
      {{ t("settings.transcriptionDeviceHelp") }}
    </p>
    <p
      v-if="failed"
      data-testid="transcription-device-error"
      role="alert"
      class="text-xs text-danger"
    >
      {{ t("settings.transcriptionDeviceSaveError") }}
    </p>
  </div>
</template>
<script setup lang="ts">
import { ref } from "vue";
import { useSettingsStore } from "@/stores/settingsStore";
import { useI18n } from "@/composables/useI18n";
import type { TranscriptionDevice } from "@/types/settings";
const store = useSettingsStore();
const { t } = useI18n();
const failed = ref(false);
async function change(event: Event) {
  const select = event.target as HTMLSelectElement;
  const value = select.value;
  if (!["auto", "gpu", "cpu"].includes(value)) return;
  failed.value = !(await store.saveTranscriptDevice(
    value as TranscriptionDevice,
  ));
  select.value = store.settings.transcript.device;
}
</script>
