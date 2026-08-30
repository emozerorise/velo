import { computed, ref } from 'vue';
import { en, type MessageKey, type Messages } from '@/locales/en';
import { th } from '@/locales/th';

export const SUPPORTED_LOCALES = [
  { code: 'en', label: 'English' },
  { code: 'th', label: 'ไทย' },
] as const;

export type Locale = (typeof SUPPORTED_LOCALES)[number]['code'];

const messages: Record<Locale, Messages> = { en, th };

// Module-level so every component shares one locale and re-renders together
// when it changes. The settings store owns what it is set to.
const locale = ref<Locale>('en');

function isSupported(value: string): value is Locale {
  return SUPPORTED_LOCALES.some((l) => l.code === value);
}

/// Falls back to English for anything unrecognised. Returns the locale that
/// was actually applied, which the caller mirrors onto `<html lang>`.
export function setLocale(value: string): Locale {
  locale.value = isSupported(value) ? value : 'en';
  return locale.value;
}

/// Look up a message, filling `{name}` placeholders from `params`. Falls back
/// to English and then to the key itself, so a gap in a translation degrades
/// to readable text rather than an empty label.
export function translate(
  key: MessageKey,
  params?: Record<string, string | number>
): string {
  const template = messages[locale.value][key] ?? en[key] ?? key;
  if (!params) {
    return template;
  }

  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match
  );
}

export function useI18n() {
  return {
    t: translate,
    locale: computed(() => locale.value),
    setLocale,
  };
}
