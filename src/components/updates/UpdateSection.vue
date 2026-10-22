<template>
  <section class="flex flex-col gap-4">
    <p class="text-fg/60">
      {{ t("updates.current", { version: updates.currentVersion || "—" }) }}
    </p>
    <template v-if="updates.manualOnly">
      <p class="text-sm leading-relaxed text-fg/70">
        {{ t("updates.manualWindows") }}
      </p>
      <a
        href="https://github.com/emozerorise/velo/releases/latest"
        class="self-start rounded-lg border border-fg/20 px-3 py-2 hover:bg-fg/10"
        @click.prevent="openReleasePage"
        >{{ t("updates.downloadLatest") }}</a
      >
      <p v-if="downloadError" role="alert" class="text-xs text-red-400">
        {{ downloadError }}
      </p>
    </template>
    <template v-else>
      <label class="flex items-center gap-3">
        <input
          type="checkbox"
          :disabled="!updates.currentVersion"
          :checked="settings.settings.updates.auto_check"
          @change="setAutoCheck(($event.target as HTMLInputElement).checked)"
        />
        {{ t("updates.auto") }}
      </label>
      <p class="text-xs leading-relaxed text-fg/50">
        {{ t("updates.autoHint") }}
      </p>
      <button
        class="self-start rounded-lg border border-fg/20 px-3 py-2 hover:bg-fg/10 disabled:opacity-40"
        :disabled="
          !updates.currentVersion ||
          updates.operating ||
          ['downloaded', 'ready'].includes(updates.phase)
        "
        @click="check()"
      >
        {{
          t(updates.phase === "checking" ? "updates.checking" : "updates.check")
        }}
      </button>
      <p v-if="checked && updates.phase === 'idle'" role="status">
        {{ t("updates.latest") }}
      </p>
      <p
        v-if="
          updates.version &&
          !['idle', 'error', 'checking'].includes(updates.phase)
        "
        class="font-medium"
      >
        {{
          t(updates.phase === "ready" ? "updates.ready" : "updates.available", {
            version: updates.version,
          })
        }}
      </p>
      <p v-if="updates.workBusy" class="text-xs text-amber-400">
        {{ t("updates.busy") }}
      </p>
      <div
        v-if="updates.error || preferenceError"
        role="alert"
        class="break-words text-xs text-red-400"
      >
        {{ t("updates.failed") }} {{ updates.error || preferenceError }}
      </div>
      <UpdateActions />
      <button
        v-if="updates.phase === 'available'"
        class="self-start text-xs text-fg/60 underline"
        @click="updates.skipVersion()"
      >
        {{ t("updates.skip") }}
      </button>
      <pre
        v-if="updates.notes && updates.phase !== 'idle'"
        class="max-h-52 overflow-auto whitespace-pre-wrap break-words font-sans text-xs leading-relaxed text-fg/70"
        >{{ updates.notes }}</pre>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-shell";
import { useUpdateStore } from "@/stores/updateStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useI18n } from "@/composables/useI18n";
import UpdateActions from "./UpdateActions.vue";
const updates = useUpdateStore();
const settings = useSettingsStore();
const { t } = useI18n();
const checked = ref(false);
const preferenceError = ref("");
const downloadError = ref("");
async function openReleasePage() {
  downloadError.value = "";
  try {
    await open("https://github.com/emozerorise/velo/releases/latest");
  } catch {
    downloadError.value = t("updates.openDownloadFailed");
  }
}
async function check() {
  if (!updates.currentVersion) return;
  await updates.checkForUpdates(true);
  checked.value = true;
}
async function setAutoCheck(enabled: boolean) {
  if (!updates.currentVersion) return;
  const saved = await settings.saveSettings({
    ...settings.settings,
    updates: { ...settings.settings.updates, auto_check: enabled },
  });
  preferenceError.value = saved ? "" : t("updates.saveFailed");
}
</script>
