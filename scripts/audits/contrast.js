/**
 * Contrast audit — S-09, `MASTER.md` §9 ("contrast below 4.5:1" is on the ban list) and §10.
 *
 * Runs inside the page, once per surface per theme, and posts every text node's ratio against
 * the background actually behind it. `src/lib/a11y/contrast.ts` holds the same arithmetic for
 * the specimen screen; the formulas are WCAG's and are short enough that two copies of them
 * cannot drift in a way that matters, whereas sharing them would mean this audit importing from
 * the application bundle it is supposed to be measuring.
 *
 * **The background is walked up the tree, not read off the element.** Almost every element in
 * this app computes to `rgba(0, 0, 0, 0)`, because the surface colour is painted by a card or a
 * pane several ancestors above. An audit that read `backgroundColor` off the text's own element
 * would compare every foreground against transparent black and report the entire application as
 * perfect, which is the most convincing way for a check like this to be worthless.
 *
 * Three things are deliberately **not** measured, each because measuring it would produce a
 * number that means nothing:
 *
 * * Text over a translucent background, where the ratio depends on what is behind it. It is
 *   reported as `skipped` rather than silently passed.
 * * Elements smaller than a pixel or clipped to nothing.
 * * Anything under a `data-a11y-exempt="<reason>"` attribute. WCAG 1.4.3 exempts incidental text
 *   — a decoration whose meaning is carried in full somewhere else on the same screen — and
 *   this audit cannot tell that from content on its own. The exemption is therefore **written
 *   into the markup with its reason** rather than kept in a list here, so it is read by whoever
 *   next edits that element and is counted in the output rather than dropped from it. There is
 *   no way to exempt something quietly.
 */

const AA_TEXT = 4.5;
const AA_LARGE = 3;

function parseRgb(css) {
  const nums = css.match(/-?[\d.]+/g);
  if (!nums || nums.length < 3) return null;
  const [r, g, b, a = 1] = nums.map(Number);
  return { r, g, b, a };
}

function luminance({ r, g, b }) {
  const lin = (c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function ratio(fg, bg) {
  const la = luminance(fg);
  const lb = luminance(bg);
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

/** Composite `over` onto `under`, both opaque-ish, so a wash on a card is measured as painted. */
function flatten(over, under) {
  const a = over.a;
  return {
    r: over.r * a + under.r * (1 - a),
    g: over.g * a + under.g * (1 - a),
    b: over.b * a + under.b * (1 - a),
    a: 1,
  };
}

/**
 * The colour actually behind `element`, composited from every translucent layer above the first
 * opaque one. Returns null when the stack never reaches an opaque colour, which only happens
 * above `<html>` and means the page has no background at all.
 */
function backgroundBehind(element) {
  const layers = [];
  for (let node = element; node; node = node.parentElement) {
    const colour = parseRgb(getComputedStyle(node).backgroundColor);
    if (!colour || colour.a === 0) continue;
    layers.push(colour);
    if (colour.a === 1) {
      return layers.reduceRight((under, over) => flatten(over, under));
    }
  }
  return null;
}

/** Text this element paints itself — not what its children paint. */
function ownText(element) {
  return [...element.childNodes]
    .filter((node) => node.nodeType === Node.TEXT_NODE)
    .map((node) => node.textContent.trim())
    .join(' ')
    .trim();
}

function isVisible(element) {
  const box = element.getBoundingClientRect();
  if (box.width < 1 || box.height < 1) return false;
  const style = getComputedStyle(element);
  return style.visibility !== 'hidden' && style.opacity !== '0';
}

const findings = [];
let measured = 0;
let exempt = 0;

for (const element of document.querySelectorAll('body *')) {
  const text = ownText(element);
  if (!text || !isVisible(element)) continue;

  const excused = element.closest('[data-a11y-exempt]');
  if (excused) {
    exempt += 1;
    findings.push({
      text: text.slice(0, 40),
      verdict: 'exempt',
      reason: excused.getAttribute('data-a11y-exempt'),
    });
    continue;
  }

  const style = getComputedStyle(element);
  const fg = parseRgb(style.color);
  const bg = backgroundBehind(element);

  if (!fg || !bg) {
    findings.push({ text: text.slice(0, 40), verdict: 'skipped', reason: 'no opaque background' });
    continue;
  }

  // WCAG's large-text threshold: 18.66px bold, or 24px. Below it, 4.5:1.
  const size = parseFloat(style.fontSize);
  const weight = Number(style.fontWeight) || 400;
  const large = size >= 24 || (size >= 18.66 && weight >= 700);
  const required = large ? AA_LARGE : AA_TEXT;

  const value = ratio(fg.a === 1 ? fg : flatten(fg, bg), bg);
  measured += 1;
  if (value < required) {
    findings.push({
      text: text.slice(0, 40),
      selector: `${element.tagName.toLowerCase()}.${[...element.classList].filter((c) => !/^s-[A-Za-z0-9_-]+$/.test(c)).join('.')}`,
      ratio: Number(value.toFixed(2)),
      required,
      size,
      colour: style.color,
      on: `rgb(${Math.round(bg.r)}, ${Math.round(bg.g)}, ${Math.round(bg.b)})`,
      verdict: 'fail',
    });
  }
}

await fetch('/__report__', {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({
    audit: 'contrast',
    scenario: window.__SHOT__.scenario,
    theme: window.__SHOT__.theme,
    total: measured,
    exempt,
    findings,
  }),
});
