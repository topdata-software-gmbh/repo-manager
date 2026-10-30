import { describe, expect, it } from 'vitest';

const sources = import.meta.glob('../../**/*.{vue,ts,css}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const palettePattern =
  /\b(?:bg|text|border|ring|from|via|to|divide|placeholder|accent|shadow|outline)-(?:(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-\d{2,3}|(?:white|black))\b/;

function appSource(): Array<[string, string]> {
  return Object.entries(sources).filter(
    ([file]) => !file.includes('__tests__') && !file.includes('.test.'),
  );
}

describe('token-only app styling', () => {
  it('does not use raw Tailwind palette classes outside the kit', () => {
    for (const [file, source] of appSource()) {
      expect(source, file).not.toMatch(palettePattern);
    }
  });

  it('does not use raw hex colours outside the kit', () => {
    for (const [file, source] of appSource()) {
      expect(source, file).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    }
  });
});
