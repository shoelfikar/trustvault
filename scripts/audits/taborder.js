/**
 * Tab-order audit — the third one, and the one the other two are structurally unable to run.
 *
 * `focus` asks whether an element *changes appearance* when it takes focus and `contrast` asks
 * whether text is readable; both reach their elements with `element.focus()`, which succeeds on a
 * `tabindex="-1"` control just as happily as on a real tab stop. So the question neither of them
 * can ask is the one a person with no pointer asks first: **is this control in the tab order at
 * all**, and in what order.
 *
 * This is the throwaway script that found `docs/keyboard-audit.md` findings 6 and 7, kept. It is
 * still not the manual walk and does not supersede it — it cannot press Tab, so it cannot see a
 * focus trap, and it runs against the harness, which stubs `invoke`. What it can do is enumerate
 * what the browser *would* treat as a tab stop, in document order, and refuse three shapes that
 * are always defects.
 *
 * **1 — a roving group with no stop.** A group of peers that all carry an explicit `tabindex`,
 * at least one of them negative, is somebody implementing a roving tab stop: one member at `0`,
 * the rest at `-1`, arrows moving between them. If *none* of them is at `0`, the whole control
 * leaves the tab order while staying visible, staying clickable, and passing the focus audit.
 * That is finding 6 exactly — `Segmented` wrote `tabindex={option.value === value ? 0 : -1}`, and
 * a `value` matching no option marked every button `-1`.
 *
 * **2 — a roving group with more than one stop.** The same signature with the opposite failure:
 * peers marked `-1` prove the intent, so two members at `0` means the pattern is half applied and
 * Tab lands inside the group twice.
 *
 * **3 — a positive `tabindex`.** It reorders the sequence away from document order, which is the
 * order everything else here measures against.
 *
 * **What it deliberately does not judge is how many stops a surface has.** Thirteen consecutive
 * sidebar rows (finding 7) is a defect against `docs/keyboard-audit.md` row 6 and a perfectly
 * ordinary navigation pattern anywhere else; no property of the DOM tells the two apart. The
 * count is *reported* per surface and the ordered list is printable with `--stops`, because that
 * is how finding 7 was seen — by reading the sequence, not by failing a rule.
 */

/** Things a person operates. A `div` that merely holds focus is not one of them. */
const INTERACTIVE = [
  'a[href]',
  'button',
  'input',
  'select',
  'textarea',
  '[contenteditable]',
  '[role="button"]',
  '[role="radio"]',
  '[role="checkbox"]',
  '[role="switch"]',
  '[role="menuitem"]',
  '[role="option"]',
  '[role="tab"]',
  '[role="link"]',
].join(',');

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
    .filter((name) => !name.startsWith('svelte-') && !/^s-[A-Za-z0-9_-]+$/.test(name))
    .join('.');
  return `${element.tagName.toLowerCase()}${classes ? `.${classes}` : ''}${label ? ` [${label}]` : ''}`;
}

/** Visible enough to be tabbed to: a hidden control is not a tab-order finding. */
function isVisible(element) {
  const box = element.getBoundingClientRect();
  if (box.width === 0 && box.height === 0) return false;
  const style = getComputedStyle(element);
  return style.visibility !== 'hidden' && style.display !== 'none' && style.opacity !== '0';
}

/**
 * Disabled in the way that removes a control from the tab order.
 *
 * `aria-disabled` does **not** — it is the pattern for a control that stays reachable and
 * announces why it will not act — so it is not treated as one here.
 */
function isDisabled(element) {
  return element.disabled === true;
}

/**
 * The tab order of a surface with a modal open is the **dialog's**, not the document's.
 *
 * `Dialog.svelte` traps Tab on the window and wraps it inside its card, so the thirty controls
 * behind the scrim are in the document's tab sequence and unreachable all the same. Measuring
 * the document would report *New item* as a 47-stop surface, of which the twenty-odd that
 * matter are buried — and it would report the background's shape as if the user could get
 * there. The scope is narrowed to what a person can actually reach.
 *
 * The narrowing **assumes the trap works**, which this audit cannot verify: it reads the DOM and
 * cannot press Tab. That the trap holds is a row in `docs/keyboard-audit.md` walked by hand, and
 * the assumption is worth stating because the honest alternative — `inert` on the background —
 * would make the DOM say what the JavaScript currently says.
 */
const modal = [...document.querySelectorAll('[role="dialog"][aria-modal="true"]')]
  .filter(isVisible)
  .pop();
const scope = modal ?? document.body;

const operable = [...scope.querySelectorAll(INTERACTIVE)].filter(
  (element) => isVisible(element) && !isDisabled(element),
);
const stops = operable.filter((element) => element.tabIndex >= 0);

const findings = [];

/* ---- Rule 3: positive tabindex --------------------------------------------- */

for (const element of operable) {
  if (element.tabIndex > 0) {
    findings.push({
      verdict: 'fail',
      kind: 'positive-tabindex',
      name: nameOf(element),
      detail: `tabindex="${element.tabIndex}" jumps the queue`,
    });
  }
}

/* ---- Rules 1 and 2: roving groups ------------------------------------------ */

/**
 * A composite widget: the roles whose whole definition is *one* tab stop with arrows inside it.
 *
 * `group` is deliberately absent from the one-stop rule below and present here: it fences a set
 * of peers off from the next set, which is all this list is used for when finding a group, but
 * ARIA does not make it a single-stop widget and failing it as one would be this audit inventing
 * a rule rather than reading one.
 */
const WIDGET_ROLES = new Set([
  'radiogroup',
  'menu',
  'menubar',
  'listbox',
  'tablist',
  'toolbar',
  'tree',
  'grid',
  'group',
]);

/** Of those, the ones ARIA defines as exactly one tab stop with the arrows inside. */
const ONE_STOP_ROLES = new Set([
  'radiogroup',
  'menu',
  'menubar',
  'listbox',
  'tablist',
  'toolbar',
  'tree',
]);

/** The elements that fence a region of the page off from the rest of it. */
const REGIONS = new Set(['NAV', 'MAIN', 'HEADER', 'FOOTER', 'ASIDE', 'SECTION', 'FORM', 'DIALOG']);

/**
 * The container whose members arrow between each other — found by climbing to the first
 * boundary, not by inspecting the members.
 *
 * Both halves of that are the result of getting it wrong first. Climbing *while every operable
 * descendant carries an explicit `tabindex`* looks like the natural rule and fails in both
 * directions on this application's own markup. It **over-climbs** out of a `role="radiogroup"`
 * whenever the element above it holds nothing else, which merged Settings' Theme and Interface
 * size into one six-member group with two stops — two correct controls reported as one broken
 * one. And it **under-climbs** in the sidebar, whose thirteen rows sit in three separate `<ul>`s
 * sharing one roving index: the `<nav>` above them also holds the footer's profile button, which
 * carries no `tabindex`, so the climb stopped at each list and reported the two lists that do
 * not hold today's stop as unreachable.
 *
 * A boundary answers both. A composite role is the author saying "this is one widget", and a
 * region is as far as a set of peers can plausibly reach: `nav` is where the sidebar's three
 * lists become one group, and `role="radiogroup"` is where Theme stops being Interface size.
 */
function groupOf(element) {
  for (let node = element.parentElement; node && node !== scope; node = node.parentElement) {
    if (WIDGET_ROLES.has(node.getAttribute('role')) || REGIONS.has(node.tagName)) return node;
  }
  return scope;
}

const groups = new Map();
for (const element of operable) {
  // Only an explicit `tabindex` marks membership. A control the author never touched is an
  // ordinary tab stop that happens to live nearby, and counting it as a peer would report every
  // region holding one roving group and one plain button as ambiguous.
  if (!element.hasAttribute('tabindex')) continue;
  const group = groupOf(element);
  const members = groups.get(group) ?? [];
  members.push(element);
  groups.set(group, members);
}

const reported = new Set();

for (const [group, members] of groups) {
  // Two conditions make this a roving group rather than a coincidence, and both have to hold:
  // more than one peer, and at least one of them held out of the tab order on purpose. Without
  // the second, a container whose controls all happen to carry `tabindex="0"` would be judged
  // against a pattern nobody was using.
  if (members.length < 2) {
    if (members[0].tabIndex < 0) {
      findings.push({
        verdict: 'fail',
        kind: 'unreachable',
        name: nameOf(members[0]),
        detail: 'tabindex="-1" with no peer to arrow from',
      });
    }
    continue;
  }
  if (!members.some((member) => member.tabIndex < 0)) continue;

  const inside = members.filter((member) => member.tabIndex >= 0);
  if (inside.length === 1) continue;

  reported.add(group);
  findings.push({
    verdict: 'fail',
    kind: inside.length === 0 ? 'unreachable-group' : 'ambiguous-group',
    name: nameOf(group),
    detail:
      inside.length === 0
        ? `${members.length} peers, every one at tabindex="-1" — the control cannot be tabbed to`
        : `${members.length} peers, ${inside.length} tab stops — Tab enters the group more than once`,
    members: members.map(nameOf),
  });
}

/* ---- Rule 4: a composite widget is one tab stop ---------------------------- */

/**
 * Rules 1 and 2 need an explicit `tabindex="-1"` somewhere to prove that a roving stop was
 * intended. A composite role needs no such proof: `menu`, `listbox`, `tablist` and the rest are
 * defined by ARIA as **one** tab stop with the arrows moving inside, so a `role="menu"` whose
 * four rows are four ordinary buttons is not an author's choice about Tab — it is the role's
 * own contract unmet, and Tab out of the third row lands somewhere the widget does not control.
 *
 * This is the rule that does not need a hand-written expectation, which is why the sidebar's
 * thirteen consecutive rows (finding 7) are *not* caught by it: a `<nav>` of buttons carries no
 * such contract. That one is a row in `docs/keyboard-audit.md`, decided by the author as D-67,
 * and no property of the DOM would have told it from an ordinary navigation column.
 */
for (const widget of document.querySelectorAll('[role]')) {
  if (!ONE_STOP_ROLES.has(widget.getAttribute('role'))) continue;
  if (!scope.contains(widget) || !isVisible(widget)) continue;
  // Already named by rule 1 or 2, which say more about the same container.
  if (reported.has(widget)) continue;

  const inside = [...widget.querySelectorAll(INTERACTIVE)].filter(
    (element) => isVisible(element) && !isDisabled(element) && element.tabIndex >= 0,
  );
  if (inside.length === 1) continue;

  findings.push({
    verdict: 'fail',
    kind: 'widget-stops',
    name: nameOf(widget),
    detail: `role="${widget.getAttribute('role')}" is one tab stop with arrows inside it, and this one has ${inside.length}`,
    members: inside.map(nameOf),
  });
}

await fetch('/__report__', {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({
    audit: 'taborder',
    // Named in the output rather than folded into the count: "14 stops" and "14 stops inside a
    // dialog" are different facts, and the second one says which surface was measured.
    modal: modal ? nameOf(modal) : null,
    scenario: window.__SHOT__.scenario,
    theme: window.__SHOT__.theme,
    // A surface with no tab stop at all is a finding of its own: it means the drive did not
    // reach the screen it was aimed at, and "0 problems" would be the most reassuring possible
    // way to have tested nothing.
    total: stops.length,
    findings,
    // The sequence, for reading rather than for failing. Printed by `--stops`.
    stops: stops.map(nameOf),
  }),
});
