export const fontChoices = [
  { id: 'sarabun', name: 'Sarabun', group: 'looped' },
  { id: 'noto_sans_thai_looped', name: 'Noto Sans Thai Looped', group: 'looped' },
  { id: 'ibm_plex_sans_thai_looped', name: 'IBM Plex Sans Thai Looped', group: 'looped' },
  { id: 'prompt', name: 'Prompt', group: 'loopless' },
  { id: 'kanit', name: 'Kanit', group: 'loopless' },
  { id: 'ibm_plex_sans_thai', name: 'IBM Plex Sans Thai', group: 'loopless' },
] as const;
export type FontFamily = 'noto_sans_thai' | typeof fontChoices[number]['id'];
export const defaultFont: FontFamily = 'noto_sans_thai_looped';
export function fontName(id: FontFamily): string {
  return fontChoices.find(font => font.id === id)?.name ?? 'Noto Sans Thai';
}
export function fontCss(id: FontFamily): string { return `"${fontName(id)}"`; }
