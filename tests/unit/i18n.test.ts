import { describe, it, expect, beforeEach } from 'vitest';
import { translate, setLocale } from '@/composables/useI18n';
import { en } from '@/locales/en';
import { th } from '@/locales/th';

describe('setLocale', () => {
  beforeEach(() => {
    setLocale('en');
  });

  it('applies a supported locale', () => {
    expect(setLocale('th')).toBe('th');
    expect(translate('transcript.title')).toBe(th['transcript.title']);
  });

  it('falls back to English for an unknown locale', () => {
    expect(setLocale('fr')).toBe('en');
    expect(translate('transcript.title')).toBe(en['transcript.title']);
  });
});

describe('translate', () => {
  beforeEach(() => {
    setLocale('en');
  });

  it('fills named placeholders', () => {
    expect(translate('toast.volume', { percent: 45 })).toBe('Volume: 45%');
  });

  it('leaves a placeholder alone when no value is given for it', () => {
    expect(translate('toast.speed')).toBe('Speed: {speed}x');
  });

  it('translates the same key in every locale', () => {
    setLocale('th');
    expect(translate('toast.muted')).toBe(th['toast.muted']);
    setLocale('en');
    expect(translate('toast.muted')).toBe(en['toast.muted']);
  });
});
