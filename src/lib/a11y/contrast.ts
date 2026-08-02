/**
 * WCAG 2.1 contrast ratios, computed from the colours the browser actually resolved.
 *
 * MASTER.md §2 states contrast figures for the dark theme. Restating them in a comment
 * would let them drift; computing them from the live tokens means the specimen page fails
 * visibly when a token changes. S-09 is checked against this.
 */

/** Resolve a CSS custom property to an `rgb(...)` string via the computed style. */
export function resolveToken(name: string, el: Element = document.documentElement): string {
  const probe = document.createElement('span');
  probe.style.color = `var(${name})`;
  probe.style.display = 'none';
  el.appendChild(probe);
  const resolved = getComputedStyle(probe).color;
  probe.remove();
  return resolved;
}

/** Parse `rgb(r g b)`, `rgb(r, g, b)`, or `rgba(...)` into 0–255 channels. */
function parseRgb(css: string): [number, number, number] | null {
  const nums = css.match(/-?[\d.]+/g);
  if (!nums || nums.length < 3) return null;
  const [r, g, b] = nums.map(Number) as [number, number, number];
  return [r, g, b];
}

/** WCAG relative luminance. */
function luminance([r, g, b]: [number, number, number]): number {
  const lin = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

/** Contrast ratio between two resolved CSS colours, 1–21. Returns NaN if unparseable. */
export function contrastRatio(fg: string, bg: string): number {
  const a = parseRgb(fg);
  const b = parseRgb(bg);
  if (!a || !b) return NaN;
  const la = luminance(a);
  const lb = luminance(b);
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

export type ContrastLevel = 'AAA' | 'AA' | 'AA-large' | 'fail';

/**
 * Grade a ratio. `ui` marks non-text boundaries (borders, focus rings), which WCAG 1.4.11
 * holds to 3:1 rather than 4.5:1 — the distinction matters here because MASTER.md's hairline
 * borders would otherwise read as failures.
 */
export function grade(ratio: number, ui = false): ContrastLevel {
  if (Number.isNaN(ratio)) return 'fail';
  if (ui) return ratio >= 3 ? 'AA' : 'fail';
  if (ratio >= 7) return 'AAA';
  if (ratio >= 4.5) return 'AA';
  if (ratio >= 3) return 'AA-large';
  return 'fail';
}
