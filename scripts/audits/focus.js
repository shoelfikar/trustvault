/**
 * Focus audit — `MASTER.md` §7 and §10, `docs/keyboard-audit.md`'s first two global rules.
 *
 * Runs **inside the page**, after the scenario has driven itself to the surface under test, and
 * posts what it finds to `/__report__`. It is loaded as text and injected rather than imported,
 * because the page it runs in is served by the harness and has no module graph of its own.
 *
 * The question it answers is the one reading CSS cannot: *does focusing this element change how
 * it looks?* A component that sets `outline: none` and replaces it with a box-shadow passes; one
 * that sets `outline: none` and replaces it with nothing fails; and the global rule in
 * `global.css` is not evidence for either, because a component's scoped selector outranks it and
 * that is exactly how a ring goes missing in one place while every other place still has one.
 *
 * The comparison is against the element's own unfocused paint, not against a fixed expectation.
 * §7 asks for a 2px accent ring and §7 also specifies a 1px inset ring for a list row, so a rule
 * written as "2px, offset 2" would fail the two surfaces that follow the design most closely.
 * What is common to every acceptable answer is that *something visible changes*, which is what
 * a keyboard user actually needs.
 *
 * **The change is looked for on the element and on its first few ancestors**, which is not a
 * loosening — it is where the ring actually is on the most common control in the application.
 * `TextField` puts `outline: none` on its `<input>` deliberately and draws the ring on the
 * `.box` around it, so that it wraps the reveal button too; measuring the input alone reported
 * the app's primary text field as ringless on every screen that has one. The ancestor that
 * carried the change is named in the output, so "the ring is two levels up" stays visible
 * rather than becoming a way for a real finding to pass.
 */

/** Everything the browser will put a caret or a focus ring on, in document order. */
const FOCUSABLE =
  'a[href],button,input,select,textarea,[tabindex]:not([tabindex="-1"]),[contenteditable]';

/**
 * The properties a focus indicator can live in. Any of them changing is a pass.
 *
 * **`outline-offset` is deliberately absent**, and its absence is the first real finding this
 * audit produced. `global.css` sets both `outline` and `outline-offset` on `:focus-visible`, so a
 * component that cancels the outline with `outline: none` and forgets to replace it still
 * inherits the offset change: focusing it moves a ring that is not drawn, `outlineOffset`
 * differs, and the element passes the check written to catch exactly that. It would have signed
 * off the command palette's ringless search field as ringed.
 */
const WATCHED = [
  'outlineStyle',
  'outlineWidth',
  'outlineColor',
  'boxShadow',
  'borderColor',
  'borderWidth',
  'backgroundColor',
  'textDecorationLine',
];

/** A stable name for a finding — enough to go and look at it, never its contents. */
function nameOf(element) {
  const label =
    element.getAttribute('aria-label') ??
    element.getAttribute('title') ??
    element.getAttribute('placeholder') ??
    (element.tagName === 'INPUT'
      ? `${element.type} input`
      : element.textContent.trim().slice(0, 32));
  const classes = [...element.classList]
    // Svelte's scoping class changes on every build; a finding named after one is a finding
    // nobody can look up twice.
    .filter((name) => !name.startsWith('svelte-'))
    .join('.');
  return `${element.tagName.toLowerCase()}${classes ? `.${classes}` : ''}${label ? ` [${label}]` : ''}`;
}

function paintOf(element) {
  const style = getComputedStyle(element);
  const paint = Object.fromEntries(WATCHED.map((property) => [property, style[property]]));
  // An outline with no style or no width paints nothing, whatever its colour says. Collapsing
  // it here means a colour that changes under a `none` style is not mistaken for a ring.
  if (paint.outlineStyle === 'none' || parseFloat(paint.outlineWidth) === 0) {
    paint.outlineColor = 'not painted';
    paint.outlineWidth = 'not painted';
  }
  return paint;
}

/** Visible enough to be walked at all: a hidden control is not a focus finding. */
function isVisible(element) {
  const box = element.getBoundingClientRect();
  if (box.width === 0 && box.height === 0) return false;
  const style = getComputedStyle(element);
  return style.visibility !== 'hidden' && style.display !== 'none' && style.opacity !== '0';
}

/** The element and the two boxes it sits in — as far up as a ring may reasonably be drawn. */
function ringCarriers(element) {
  const chain = [element];
  for (let node = element.parentElement, depth = 0; node && depth < 2; node = node.parentElement) {
    chain.push(node);
    depth += 1;
  }
  return chain;
}

const findings = [];
const elements = [...document.querySelectorAll(FOCUSABLE)].filter(isVisible);

for (const element of elements) {
  // Disabled controls cannot take focus and are not required to show a ring. They are counted,
  // because "everything on this surface is disabled" is itself worth seeing in the output.
  if (element.disabled) {
    findings.push({ name: nameOf(element), verdict: 'disabled' });
    continue;
  }

  const carriers = ringCarriers(element);
  // Blur whatever holds focus first. Without this, the one autofocused control on a screen —
  // the lock screen's password field, onboarding's name — is measured *while already focused*,
  // so focusing it changes nothing and it is reported as the only ringless control on the
  // surface. The first two findings this audit ever produced were both that, and both were the
  // audit's fault rather than the app's.
  document.activeElement?.blur?.();
  const before = carriers.map(paintOf);
  // `focusVisible: true` asks the browser to treat this as a keyboard focus rather than a
  // programmatic one, which is the whole difference between `:focus` and `:focus-visible`.
  // Without it every element in the app would fail, and it would fail for a reason that has
  // nothing to do with the application.
  element.focus({ focusVisible: true });
  const focused = document.activeElement === element;
  const after = carriers.map(paintOf);
  element.blur();

  if (!focused) {
    findings.push({ name: nameOf(element), verdict: 'unfocusable' });
    continue;
  }

  const at = before.findIndex((paint, index) =>
    WATCHED.some((property) => paint[property] !== after[index][property]),
  );
  findings.push(
    at === -1
      ? { name: nameOf(element), verdict: 'no-indicator' }
      : {
          name: nameOf(element),
          verdict: 'ok',
          on: at === 0 ? 'self' : nameOf(carriers[at]),
          changed: WATCHED.filter((property) => before[at][property] !== after[at][property]),
        },
  );
}

await fetch('/__report__', {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({
    audit: 'focus',
    scenario: window.__SHOT__.scenario,
    theme: window.__SHOT__.theme,
    // A surface with no focusable element at all is a finding of its own: it means the drive
    // did not reach the screen it was aimed at, and an audit that reported "0 problems" for it
    // would be the most reassuring possible way to test nothing.
    total: elements.length,
    findings,
  }),
});
