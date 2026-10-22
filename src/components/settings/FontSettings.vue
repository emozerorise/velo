<template>
  <section class="space-y-5" aria-labelledby="font-heading">
    <header>
      <h4 id="font-heading" class="text-lg font-semibold">{{ t('font.title') }}</h4>
      <p class="mt-1 text-fg/60 leading-relaxed">{{ t('font.description') }}</p>
    </header>
    <div class="font-groups grid gap-5" role="radiogroup" :aria-label="t('settings.tab.typography')">
      <div v-for="group in groups" :key="group" class="min-w-0">
        <h5 class="font-semibold mb-2">{{ t(group === 'looped' ? 'font.looped' : 'font.loopless') }}</h5>
        <label
v-for="font in fontChoices.filter(item => item.group === group)" :key="font.id"
          class="flex items-center gap-3 rounded-xl px-3 py-3 cursor-pointer border border-transparent focus-within:ring-2 focus-within:ring-accent"
          :class="draft === font.id ? 'bg-accent/10 border-accent/20' : 'hover:bg-fg/5'">
          <input
type="radio" name="reading-font" :value="font.id" :checked="draft === font.id"
            :disabled="store.fontSaving" :data-testid="`font-${font.id}`" class="accent-accent shrink-0"
            @change="select(font.id)" />
          <span class="min-w-0 leading-relaxed">
            <span class="block text-[13px]">{{ font.name }}</span>
            <span class="block text-base text-fg/65" :style="{ fontFamily: fontCss(font.id) }">{{ t('font.sample') }}</span>
          </span>
        </label>
      </div>
    </div>
    <div class="rounded-xl bg-fg/5 p-4">
      <div class="flex flex-wrap justify-between gap-2 text-xs text-fg/65 mb-3">
        <span>{{ t('font.preview') }}</span><span data-testid="font-preview-name">{{ fontName(draft) }}</span>
      </div>
      <div data-testid="font-preview" class="rounded-lg bg-surface p-4 reading-text" :style="{ fontFamily: fontCss(draft) }">
        <h5 class="font-semibold text-lg mb-2">{{ t('font.previewHeading') }}</h5>
        <p>{{ t('font.previewBody') }}</p>
        <p class="font-mono text-xs text-fg/60 mt-3">Default · 01:24</p>
      </div>
    </div>
    <p class="text-xs text-fg/65 leading-relaxed">{{ t('font.scope') }}</p>
    <p v-if="message" :role="failed ? 'alert' : 'status'" class="text-sm" :class="failed ? 'text-danger' : 'text-success'">{{ message }}</p>
    <div class="flex flex-wrap justify-end gap-3">
      <button
data-testid="font-reset" class="rounded-lg border border-fg/20 px-4 py-2.5 hover:bg-fg/5 focus-visible:ring-2 focus-visible:ring-accent"
        :disabled="store.fontSaving" @click="select(defaultFont)">{{ t('font.reset') }}</button>
      <button
data-testid="font-apply" class="rounded-lg bg-accent text-white px-4 py-2.5 disabled:opacity-50 focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2"
        :disabled="store.fontSaving || draft === store.settings.general.font_family" @click="apply">{{ t(store.fontSaving ? 'font.saving' : 'font.apply') }}</button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useSettingsStore } from '@/stores/settingsStore';
import { useI18n } from '@/composables/useI18n';
import { defaultFont, fontChoices, fontCss, fontName, type FontFamily } from '@/utils/fonts';
const store = useSettingsStore();
const { t } = useI18n();
const groups = ['looped', 'loopless'] as const;
const draft = ref<FontFamily>(store.settings.general.font_family);
const message = ref('');
const failed = ref(false);
function select(font: FontFamily) { draft.value = font; message.value = ''; }
async function apply() {
  failed.value = !(await store.saveFont(draft.value));
  message.value = t(failed.value ? 'font.error' : 'font.saved');
}
</script>

<style scoped>
@media (min-width: 600px) {
  .font-groups { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
</style>
