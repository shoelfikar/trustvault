# Keyboard audit — R-30, S-08

Every reachable surface operable by keyboard alone, with no pointer. S-08 is the measurement; this
document is what it is measured against.

**Started 2026-08-05, at the beginning of Phase 3 and not at the end.** That is deliberate and it is
the only interesting decision in this file. Filled in at the end, an audit records whatever the
implementation happened to do — every box gets ticked, because the boxes were written by looking at
the thing they are auditing. Written first, it is a specification: a row that cannot be ticked is a
bug with a name, found while the surface is still being built.

So **an unticked box here is not a to-do, it is a failing test.** The phase does not exit with one
open, and a row that turns out to be wrong gets corrected in place with the reason, not quietly
deleted.

## How to run it

With the pointer physically unavailable — unplug the mouse or sit on your hands; "not using it" is
not the same test. Then walk every row. A row passes when the action completes **and** the focus
ring is visible at every step, because a surface that is operable but invisible is one nobody can
operate.

Screen readers are **out of scope for v1** and that is a scope decision, not an oversight: R-30 asks
for keyboard operability and S-08 measures it. `aria-*` correctness is tracked where it makes the
keyboard work — a `role` that breaks arrow navigation is a finding here — and not otherwise.

## Global rules

These hold on every surface, so they are checked once and then assumed.

Six of the seven are ticked by `scripts/a11y.mjs` or by a grep over the source — a machine can
answer them, and a machine re-answers them on every build rather than once on the day somebody
read the code. The one that is not ticked is the one only a person can answer, and it says so.

- [x] **`:focus-visible` on every interactive element**, distinct from the selection ring — a
      selected list row and a focused one must not look the same, or Tab is untrackable inside the
      list. `MASTER.md` §7 and §10. **Measured 2026-08-07** by `npm run a11y -- --audit focus`:
      every focusable element on all 17 scenarios in both themes changes appearance when it takes
      keyboard focus. It found one real failure doing it — the command palette's search field had
      cancelled the global ring and replaced it with nothing since Phase 2, invisible because the
      palette opens with that field already focused. Selection is a fill and focus is a ring, in
      both lists, which is §7's own wording
- [x] **Tab order follows visual order** on every surface. No positive `tabindex` anywhere in the
      codebase; a positive value reorders the whole document, not the component it appears in.
      **Measured on every build since 2026-08-14** by `npm run a11y -- --audit taborder`, which
      replaced the grep this line used to carry: a grep reads the source and cannot see a control
      that left the tab order because the *value* it was compared against went missing, which is
      how finding 6 survived. The audit enumerates what the browser would treat as a tab stop, in
      document order, and fails three shapes — a roving group with no stop, a roving group with
      two, and any positive `tabindex`. *Visual order* is still the manual pass below, because DOM
      order and painted order are the same thing only until something is absolutely positioned
- [x] **No keyboard trap.** From any focused element, Tab and Shift+Tab eventually leave — except
      inside a modal, where the trap is the point and Esc is the exit. Nothing in `src/` calls
      `preventDefault` on Tab outside `Dialog.svelte`, which is the modal case
- [x] **Every dialog traps focus, restores it on close**, and closes on Esc — the three together,
      because restoring focus to `document.body` passes the first two and strands the user at the
      top of the page. All three are in `Dialog.svelte` and every overlay in the app is built on
      it. **The restore was the one that was missing** and it was added 2026-08-07: the trap and
      Esc were there from Phase 2, and closing any dialog dropped focus on `<body>`
- [x] **Nothing is reachable only by hover.** Row actions that appear on hover must also appear on
      focus; this is the single most likely finding in the whole audit, and the detail pane's
      toolbar and the item list's row actions are where to look first. **It is not a finding**:
      no rule in `src/` gates `opacity`, `display` or `visibility` on `:hover`. The row actions
      are drawn always — the design never adopted the hover-reveal pattern, which is the reason
      the likeliest finding in the list is absent
- [x] **No handler is bound to a bare printable key** outside a text field. `/` to focus search is
      the usual offender: it eats the character in every field that forgot to stop propagation.
      Every shortcut in the app carries a modifier — ⌘K, ⌘N, ⌘G, ⌘L — and the only bare keys read
      anywhere are Escape, Enter, Tab and the arrows
- [x] **`prefers-reduced-motion` honoured** — `MASTER.md` §10. Listed here because a focus ring
      that animates into place is a focus ring you cannot follow. `tokens.css` drops every
      duration token to 80 ms under the query, which is `MASTER.md` §5's own rule rather than
      zero, and the two surfaces with animation of their own — the lock scrim and the dialog
      entrance — each answer the query a second time

## Surfaces

Grouped as the design groups them. The exit gate says **15 surfaces**; the rows below are
enumerated from `src/lib/` rather than from that number, and **reconciling the two is owed at the
gate** — if the count disagrees, the source is right about what exists and the gate line needs a
list rather than a total. Recorded now so the disagreement is found at the audit and not argued
about on the last day.

**The count is 19 and the reconciliation is done: the gate line needs a list.** The four the
gate's "15" does not cover are row 8a (the one-time code row, R-20, decided in Phase 3), row 9
(empty states, which are a surface per view rather than one screen), row 18 (the edit dialog,
D-48 — the prototype draws no edit surface) and row 19 (the import dialog, D-42/D-59 — the
design predates the importer). None of them is a surface the design drew and we skipped; all
four exist because a requirement needed one, which is the direction the disagreement was always
going to point.

**All twenty-two boxes are ticked. S-08 is met.** These rows are
what a person finds with the pointer unplugged: whether Tab lands where the eye is, whether a
dialog gives focus back to the row it was opened from, whether swallowing Enter in the tag field
reads as broken to somebody who does not know why it happens. `scripts/a11y.mjs` answers the
global rules above on every build and it cannot answer one of these — `element.focus()` is not
Tab, and no script can tell you that the order it produced is the order you were reading in.

**The last three closed on 2026-08-15**, and how each one closed is the record worth keeping.
Row 12 was **re-worded** to the behaviour that exists rather than ticked against a clause about a
dialog nobody built — D-73, the author's call, the same shape as D-64 and D-67. Rows 20 and 21
were **re-walked on a rebuilt binary**: they had been held because `target/release/trustvault` was
built 2026-08-08 at 12:11 and D-70's profile landed at 18:13 the same day, so no binary on this
machine had ever carried the surface those two rows describe. The rebuild is what made them
walkable, and it took five minutes rather than a second sitting.
Details are in the walk record below.

### Before unlock

| # | Surface | Must be operable by | Done |
|---|---------|--------------------|------|
| 1 | Onboarding step 1 — name & location | Tab through name and path, Enter advances | [x] |
| 2 | Onboarding step 2 — master password | Type, Tab to confirm, strength meter reachable but not focus-stealing, Enter advances only when valid | [x] |
| 3 | Onboarding step 3 — recovery kit | Copy and Save buttons focusable; the acknowledgement checkbox toggles with Space; Enter finishes | [x] |
| 4 | Lock screen | Password field focused **on mount**, Enter unlocks, the recovery link is Tab-reachable | [x] |
| 5 | Recovery dialog, both steps | Esc closes from either step; Enter advances; the reissued kit's copy button is reachable | [x] |

Row 4's "focused on mount" is the one autofocus in the application that is unambiguously right: the
lock screen exists to receive a password and has one field. Everywhere else, autofocus is a finding.

### The shell

| # | Surface | Must be operable by | Done |
|---|---------|--------------------|------|
| 6 | Sidebar — views, tags, vault switcher trigger | Tab in, ↑/↓ between entries, Enter selects | [x] |
| 7 | Item list | ↑/↓ move selection, Home/End jump, Enter opens in the detail pane, type-ahead is **not** implemented (⌘K is the search) | [x] |
| 8 | Detail pane — fields and toolbar | Tab through fields; per-field Reveal and Copy reachable **without hover**; Edit and Delete in the toolbar | [x] |
| 8a | Detail pane — one-time code row | The **Copy code** button is a Tab stop like every other copy in the box, and the remaining seconds are readable as text rather than only as the arc — a countdown drawn in colour and geometry alone is unreadable to the people this audit is for | [x] |
| 9 | Empty states — every list | The primary action is focusable and is the first stop after the list itself | [x] |
| 10 | Titlebar / toolbar chrome | Search trigger, New item, and the lock button all Tab-reachable | [x] |

Row 7 says what is **not** built as well as what is: type-ahead in a list competes with ⌘K for the
same keystrokes and would swallow the first letter of every shortcut a later phase adds.

### Overlays

| # | Surface | Must be operable by | Done |
|---|---------|--------------------|------|
| 11 | Command palette | ⌘K/Ctrl+K opens from anywhere, Esc closes, ↑/↓ navigate, Enter copies the password, ⇧Enter opens the item — R-16 | [x] |
| 12 | New-item dialog | Type chips selectable with ←/→ and Space; every type's fields Tab-reachable; **Generate** fills the password field in place, reveals it, and leaves focus on it — re-worded 2026-08-15, **D-73**; the **New tag** field takes Enter to add a tag and does **not** submit the item; the **2FA switch** takes Space and Tab from it lands in the seed field it just revealed | [x] |
| 13 | Delete confirmation | Focus lands on **Cancel**, not Delete; Esc cancels; Enter activates whatever is focused and nothing else; for a **vault**, the confirm field is reachable and the Delete button stays disabled until the typed name matches — R-18 | [x] |
| 14 | Generator dialog | Length slider on ←/→ (and Home/End), the four set toggles on Space, Regenerate and Copy reachable — R-15 | [x] |
| 15 | Vault switcher | ↑/↓ between vaults **including the open one**, Enter switches, Tab within a row reaches **Leave**, Esc closes — R-22 | [x] |
| 16 | Settings | Every control reachable in visual order; the segmented controls on ←/→; **UI scale changes do not move focus**; the **Start at login** toggle takes Space, and when the platform refuses the write the toggle returns to its old position with the reason announced — R-21 | [x] |
| 17 | Watchtower | The findings list is a list: ↑/↓ and Enter to the offending item | [x] |
| 18 | Edit-item dialog | Every field row Tab-reachable in display order; **Replace** on a secret row reachable without hover, and Tab from it lands in the input it just opened; Remove and Add field reachable; the **New tag** field takes Enter as "add this tag" — R-17 | [x] |
| 19 | Import dialog | Esc closes from the intro **and** from the report; Choose file… reachable and the **native picker takes over from there**, so the keyboard path leaves the app and must come back to a focused dialog; Import stays disabled until a preview is on screen; the report's refusal list scrolls with the keyboard alone, not only with a wheel — R-29 | [x] |
| 20 | Profile popover | Enter or Space on the footer row opens it and focus lands on the **first row, not the header**; ↑/↓ wrap between the four rows; Esc closes and focus returns to the footer row that opened it; the ⌘, printed beside *Settings* actually reaches Settings — D-70 | [x] |
| 21 | Edit-profile dialog | Focus lands in **Full name**; Tab reaches Email, Cancel and Save in that order; Enter in either field saves rather than doing nothing; Esc cancels and focus returns to whatever opened it — the popover row **or** the Settings card's button, which are two different return paths — D-70 | [x] |

Rows 18 and 19 were added 2026-08-06 with the surfaces themselves, and rows 20 and 21 on
2026-08-08 with D-70's, which is why this table's count is now **21 against the gate's 15**. None
of the four is drawn in the prototype as a surface we then built: 18 and 19 exist because a
requirement needed them, and 20 and 21 because D-70 gave the design's own profile pixels something
to be. That widens the reconciliation this section already owed rather than changing its shape —
the source is right about what exists, and the gate line needs a list rather than a total.

Row 20's "first row, not the header" is the finding it is written against. The header is a `div`
with an avatar and two lines of text and takes no focus at all, so a menu that opened with focus
on the container would leave ↑/↓ doing nothing until Tab was pressed first — the failure looks
exactly like a menu that does not respond to the keyboard.

Row 21's two return paths are the reason it is a row rather than a clause on row 13. The same
dialog is opened from the popover and from Settings, and the popover **is gone by the time the
dialog closes** — so the restore has a dead trigger on one of the two paths and a live one on the
other. `Dialog.svelte`'s `isConnected` guard is what makes the dead one fall back rather than
silently focus `<body>`, and this row is where somebody checks that it does.

Row 19 carries the only line in this table that is **not about our own focus handling**. The native
file dialog is the operating system's, not ours: it takes focus, it is keyboard-operable or not
according to the platform, and what has to be checked here is the return — a dialog that comes back
with focus on `body` leaves the user who just chose a file with nothing selected and no visible
reason. It is also the first row that cannot be checked in the screenshot harness at all, because
the harness has no file chooser to open.

Row 18's "Tab from Replace lands in the input it just opened" is the finding waiting to happen:
pressing Replace swaps a disabled input for an editable one in the same position, and the browser
has no reason to move focus there on its own.

The tag clause on rows 12 and 18 is there because both dialogs deliberately **swallow Enter** in
one field. A dialog's Enter submits it, and a tag being typed is not a finished item — without the
interception, adding a tag would save a half-filled form, which is the more expensive of the two
surprises. Whether it reads as broken to somebody who does not know that is exactly what the manual
pass is for.

Row 13 is a requirement, not a preference. A destructive dialog that opens with the destructive
button focused converts the Enter keypress that opened it into a confirmation, which is how a
keyboard user deletes something they were only reading about.

Row 15's "including the open one" was written **into the switcher** on 2026-08-06 rather than
found by this table later, which is what the checklist is for. The open vault's row was `disabled`
first, which is the obvious way to say "you are already here" and is wrong for exactly one reason:
a disabled button cannot take focus, so ↑/↓ stops dead on the row the user is standing in and
reads as the arrow key having failed. It is `aria-disabled` instead — focusable, announced as
unavailable, and a no-op when activated. The same shape is worth watching for anywhere else a
"current" row is drawn.

Row 16's parenthetical is the one that will be missed: UI scale re-renders the root, and a naïve
implementation drops focus to `body` — leaving the user who just changed a setting at the top of the
application with no way back except Tab.

## Findings

Filled in as they are found, cleared as they are fixed. A finding is deleted only when the fix is
committed; a finding that stops reproducing without a fix gets a line saying so, because
intermittent keyboard bugs are usually focus-order bugs that depend on what was focused before.

**Three found on 2026-08-07, in the global-rules pass. All three are fixed and the fixes are
committed**, so by the rule above they would be deleted — they are kept here because each one
says something about how it was missed, and the manual pass below has not run yet.

1. **A dialog gave focus back to nothing.** `Dialog.svelte` trapped focus and closed on Esc from
   Phase 2 and never restored focus to the element that opened it, so closing any overlay left
   the user at the top of the application. This is the failure global rule 4 is worded against —
   "the three together" — and it survived two phases because the two visible parts worked.
   Fixed: the opener is captured in the component body, before the effect that moves focus in.
2. **The command palette's search field had no focus indicator.** It cancelled the global ring
   with `outline: none` and replaced it with nothing. Invisible in use because the palette opens
   with that field already focused, so the only way to see it is to Tab to a result and back.
   Found by `scripts/a11y.mjs`, not by reading — and reading would not have found it either, as
   the CSS looks deliberate. Fixed: the ring is on the row, as `TextField` does it.
3. **The audit tool passed it anyway, the first time.** `outline-offset` still changed when the
   outline itself was `none`, so "something about the focus paint differs" was satisfied by a
   ring that is not drawn. Fixed in the tool, and it is the reason the tool now collapses an
   unpainted outline before comparing.

**One found on 2026-08-08, in the manual pass, on the first surface walked.** It is two defects
sharing a screen, and the row fails on either one alone.

4. **Step 3 of onboarding has no Enter path and receives no focus.** Reported by the author
   walking row 3 with the pointer unplugged: neither Tab nor Enter does what the row asks.
   Reading the source gives two independent causes, and the second is the one that generalises.

   *Enter finishes nothing.* `onenter` is implemented in `TextField.svelte:88` as an `onkeydown`
   on the input itself, so Enter advances a step **only where a text field is focused**. Steps 1
   and 2 have one; step 3 has none — it is a key, two print buttons, a checkbox and the CTA. There
   is no `<form>` and no keydown handler on the screen, so on step 3 Enter reaches nothing at all.
   The row's "Enter finishes" was never implemented; it reads as satisfied because the two steps
   before it satisfy it by accident of having an input.

   *Focus is on `document.body`.* Steps 1 and 2 each carry `autofocus`
   (`Onboarding.svelte:250`, `:281`); step 3 carries none. When `step` becomes 3 the focused
   password field is removed from the DOM, and focus falls to `body` — so the first Tab restarts
   from the top of the document rather than from the kit the user is reading. This is **finding 1
   in a second place**: `Dialog.svelte` was fixed on 2026-08-07 for exactly this, and the fix was
   made to the dialog rather than to the pattern, so the step transition kept the bug.

   Neither is reachable by `scripts/a11y.mjs`, and the reason is worth keeping: the tool asks
   whether every focusable element *can* take focus, and both of these are about what happens
   **between** two renders. Step 3's elements are all perfectly focusable. Nothing focuses them.

   **Fixed 2026-08-08 as D-68.** Focus lands on the step's first control when `step` becomes 3,
   which is where `autofocus` puts it on the two steps before; and Enter is handled on the step,
   ignoring Enter on a `<button>` so that Enter on *Print* prints instead of printing **and**
   finishing — finishing clears the recovery code while the print dialog still holds it. **Row 3
   stays unticked**: the fix has not been walked, and the row is what says whether it worked.

5. **Every palette command that opens an overlay is cancelled by the palette closing.** Reported
   by the author as "New item from the search does nothing". It is not a keyboard defect and does
   not belong to this audit's subject at all — **the mouse path is identically broken** — which is
   why it is written up here as the walk that found it rather than as a row that failed.

   `Shell.svelte` holds one `overlay` state for every overlay in the application, and the palette's
   command row runs `command.run()` and then `onclose()` back to back
   (`CommandPalette.svelte:167-170`). For *New item* that is `overlay = 'add'` immediately
   overwritten by `overlay = 'none'`: two synchronous assignments to the same state, last write
   wins, and the dialog the user asked for never renders. **Generate password** is broken the same
   way (`overlay = 'generator'`). *Lock vault*, *Watchtower* and *Settings* are not, and that is
   the reason it survived: they route through `onlock` and `onview`, which do not touch `overlay`,
   so three of the five command rows work and the palette looks alive.

   The empty-state action has the same two lines in the same order
   (`CommandPalette.svelte:266-269`), and it is the worse of the two: its comment explains that it
   exists because a query matching no item has also filtered the *New item* command row away, so
   it is **the only way** to act on something just found missing. The only way was also the
   broken way.

   **Fixed 2026-08-08 as D-66**, in `Shell.svelte` rather than in either call site: `closeOverlay`
   clears the overlay only when it is still the one on screen, so a handler that navigated
   somewhere keeps where it went. Reordering the two calls would have fixed these two rows and
   left ordering as the thing that decides. **Row 11 is still unticked** — its own four clauses
   have not been walked, and the fix needs the walk to confirm it in the app.

   How it was missed is the part worth keeping. `⌘N` and `⌘G` reach the same dialogs through the
   shell's shortcut handler without going near the palette, so both dialogs work everywhere a
   person would normally open them. The screenshot harness photographs each overlay by setting
   `overlay` directly, so it has a picture of a dialog that cannot be opened this way — the same
   shape as **D-47**, where `ipc_session.rs` drove command bodies past the argument decoding that
   was broken. A harness that constructs the state under test cannot see a transition that
   destroys it.

6. **`Segmented` leaves the tab order entirely when its `value` matches no option.** Written as
   `tabindex={option.value === value ? 0 : -1}`, so a value outside the list marks **no** option
   as the tab stop and all of them as `-1`: the control stays visible, stays clickable, still
   passes the focus audit — `element.focus()` reaches a `-1` button perfectly well — and cannot be
   tabbed to. Found by measuring the tab order rather than by reading, and it was **already
   happening**: `scripts/harness.mjs`'s settings fixture was missing `ui_scale` and
   `launch_at_login` (added by D-57 and D-56, never backfilled), so Interface size had been
   keyboard-unreachable in every screenshot and every a11y run since. Both fixed 2026-08-08 — the
   fixture is exhaustive with a note saying to keep it so, and the component falls back to the
   first option, which is what ARIA prescribes for a radiogroup with nothing checked.

   The fixture half is the more useful lesson: the Settings surface had been measured against a
   settings object **the host cannot produce**, and nothing said so. It is the third member of
   the D-47 family in this phase.

7. **The sidebar is thirteen consecutive tab stops, and row 6 says it should be one.** Measured
   in the `settings` scenario: 24 tab stops, of which 0–2 are the titlebar, **3–15 are sidebar
   rows**, and 16–23 are the Settings pane. So reaching the first Settings control from the
   titlebar's Settings button takes **fifteen Tab presses, thirteen of them through the sidebar**
   — which is what "Tab never gets into Settings" looks like from a chair.

   Row 6 reads *"Tab in, ↑/↓ between entries, Enter selects"*, which describes one tab stop with
   arrows moving inside it — the pattern `Segmented` uses. `Sidebar.svelte` has **no keydown
   handler at all**, so the arrow half is not implemented and every row is its own stop. Row 6
   is recorded as passed above because Tab does enter and Enter does select; the clause between
   them was never exercised.

   **This was a wording-or-implementation decision and it was the author's**, the same shape as
   D-64 and as row 12's generator clause. **Decided 2026-08-08 as D-67: the implementation
   changes and row 6 stands as written.** The sidebar is one roving tab stop, ↑/↓ move focus
   without selecting, Enter selects. Re-measured: **13 tab stops in the whole window, down from
   24**, and the first Settings control is four Tab presses from the titlebar rather than fifteen.

   Row 6 is **un-ticked** as a consequence, and that is the point rather than a cost: it passed
   against markup that no longer exists, and the clause it never exercised is the one that
   changed.

8. **The vault existed before its recovery kit had been recorded.** Reported while walking row 3,
   and like finding 5 it is not a keyboard defect — it is a hole in **R-07**, found because
   somebody was reading the screen rather than the code.

   `create_vault` wrote the file at the end of step 2 and `remember_vault` ran immediately, so a
   user who closed the window while reading the kit on step 3 owned a `.tvault` whose kit had
   never been written down. R-07 shows it exactly once, the command's own comment says there is
   no command to fetch it again, and the remembered path (D-40) sent the next launch to a lock
   screen. That vault has no recovery route for the rest of its life, and nothing in the product
   knew — the one requirement whose whole purpose is *what happens when the password is gone*,
   defeated by closing a window.

   **Fixed 2026-08-08 as D-69**: `create_vault` stops at memory and a new `commit_vault` is what
   writes, called by step 3's acknowledgement. Nothing is on disk until the box is ticked.
   **Row 3 stays unticked** — the row now exercises a write as well as a keypress.

9. **Enter did not tick the acknowledgement checkbox.** Recorded first as a question rather than
   a defect, because row 3 asks for the opposite in its own words — *"the acknowledgement
   checkbox toggles with **Space**; **Enter** finishes"* — and a checkbox that toggles on Space
   and ignores Enter is native behaviour in every browser. **Reported twice from the walk, which
   is the answer**: correct by the row's letter, and a dead key at the keyboard, on the one
   control the whole step exists to collect.

   **Fixed 2026-08-08.** Enter toggles it, exactly as Space does. It does **not** finish when the
   box is already ticked: the same key on the same control doing two different things depending
   on state is worse than the dead key it replaces.

   **Row 3 is unchanged**, and this is the case where that is the right outcome rather than the
   lazy one. Both its clauses are now more true than before: Space still toggles, and Enter still
   finishes — from the CTA, which is where Tab lands the moment the box is ticked and the button
   stops being disabled. Nothing about the row needed re-wording, unlike row 6 (D-67) and row 12,
   because the row was describing a keyboard that works and the implementation was the half that
   did not.

10. **The profile popover is four tab stops, and `role="menu"` promises one.** Found 2026-08-14 by
    the tab-order audit on its first full sweep, on a surface built six days earlier (D-70) and
    never walked. Nobody reported it: with a pointer it is perfect, and with a keyboard the four
    rows work — ↑/↓ move between them, Esc closes, Enter chooses. What is broken is **leaving**.
    Tab from *Settings* does not close the menu and does not stay in it; it steps to *Lock vault*
    and then out into whatever sits behind the open popover, which the menu neither controls nor
    knows about.

    This is the first finding here that is a defect against a **specification rather than against
    a row**. Rows 6 and 12 needed the author to decide what the surface should do; `role="menu"`
    already says — ARIA defines a menu as one tab stop with the arrows inside it — so there was
    nothing to decide and the fix is the implementation catching up to the role it had already
    claimed. It is also why the audit could find it at all: the sidebar's thirteen stops
    (finding 7) fail no such contract, and no property of the DOM tells that shape from an
    ordinary navigation column.

    **Fixed 2026-08-14**, in `Sidebar.svelte`'s pattern: one roving `tabindex`, the arrows moving
    it, Enter choosing. **Confirmed in the app on 2026-08-15**: row 20 was walked on a rebuilt
    binary with the pointer unplugged, and one Tab leaves the menu rather than stepping through
    *Lock vault* into what sits behind the popover. The audit found it; the walk is what says it
    is fixed, and those are deliberately two different events.

_Every finding in this list is now confirmed fixed by a walk rather than by a commit. Findings 4,
5, 8 and 9 by the re-walks of 2026-08-14 (rows 3, 6, 11), and finding 10 by row 20 on 2026-08-15.
Ten findings, all closed, and **the walk found four of them** — which is the argument for having
run it at all rather than trusting three passing audits._

**On findings 6, 7 and 10**: the throwaway script that found the first two is now
`scripts/audits/taborder.js`, the third audit beside `focus` and `contrast`, and finding 10 is
what it returned on its first full sweep. It is **not** pressing Tab and does not supersede this
table — it cannot see a focus trap, and it runs against the harness, which stubs `invoke`. What it
can do is answer "is this control in the tab order, once", which is the one question the other two
are structurally unable to ask: `element.focus()` reaches a `tabindex="-1"` control perfectly well.

Two things about it are worth knowing before reading its output. It measures **inside the modal**
when one is open, because `Dialog.svelte` traps Tab — so *New item* reports 22 stops rather than
the document's 47, and the narrowing assumes a trap this audit cannot verify. And it deliberately
does not judge **how many** stops a surface has: that is the row's job, and it is the half of
finding 7 no script could have decided.

## Manual pass — complete

**Opened 2026-08-08 by the author.** Recorded as it arrives rather than at the end, for the reason
the preamble gives: an audit written up afterwards records what the implementation turned out to
do.

- **The four global shortcuts fire and open their surfaces** — ⌘L lock, ⌘K search, ⌘N new item,
  ⌘G generate. Reported working. **This ticks no row**, and the gap is worth naming rather than
  rounding up: opening a surface is the first clause of rows 11, 12 and 14 and the whole of none
  of them, and ⌘L has no row at all — it is global rule 6's example, already ticked by grep. What
  these four establish is that the shortcut layer works, which is what makes the rest of the pass
  possible; a surface that cannot be opened cannot be walked.

Still owed on the three rows this touches, taken from their own wording — **all three are closed
now**: rows 11 and 14 were walked in full on 2026-08-14, and row 12 on 2026-08-15 once its first
listed clause had been re-worded to the surface that exists (D-73). Kept as written, because it is
the list that turned "the shortcut works" into three rows nobody could yet tick:

| Row | Opened by | Still to check |
|---|---|---|
| 11 Command palette | ⌘K ✓ | Esc closes, ↑/↓ navigate, Enter copies the password, ⇧Enter opens the item |
| 12 New-item dialog | ⌘N ✓ | Type chips on ←/→ and Space; every type's fields Tab-reachable; the generator opens from inside it **and returns focus**; the New tag field takes Enter without submitting; the 2FA switch takes Space and Tab lands in the seed field it just revealed |
| 14 Generator dialog | ⌘G ✓ | Length slider on ←/→ and Home/End; the four set toggles on Space; Regenerate and Copy reachable |

Row 12's "the generator opens from inside it and returns focus" is the one to watch while both are
fresh: ⌘G from the shell and the generator opened from **inside** the New-item dialog are two
different focus stories, and only the second can strand you — it is a dialog over a dialog, and
`Dialog.svelte`'s restore was written for one opener, then fixed on 2026-08-07 (finding 1 above).

### Rows walked

| Row | Result | What was observed |
|---|---|---|
| 3 Onboarding step 3 — recovery kit | **FAIL, fixed, awaiting re-walk** | Neither Tab nor Enter operates the screen. Finding 4 — two causes, one per clause: there is no Enter path on a step without a text field, and the step transition leaves focus on `document.body`. The row's third clause ("Enter finishes") was never implemented rather than broken. Fixed 2026-08-08 (**D-68**) and **not re-walked**. The same walk then found finding 8 on this screen, which is not a keyboard defect at all |
| 6 Sidebar | **passed, then re-opened** | Passed on the walk — Tab entered and Enter selected. Finding 7 then showed the middle clause had never been implemented, and the fix (**D-67**) changed the surface underneath the tick, so the row goes back to unwalked. A row ticked against code that no longer exists is worse than an empty one |
| 7 Item list | **PASS** | ↑/↓, Home/End, Enter opens in the detail pane |
| 8 Detail pane | **PASS** | Fields Tab through; Reveal and Copy reachable without hover; Edit and Delete in the toolbar |
| 8a One-time code row | *not yet walked* | Separate box from row 8, and only visible on an item that carries a one-time code — the row most likely to be skipped by walking the detail pane of an item that has none |
| 9 Empty states | **PASS** | The primary action is focusable and is the first stop after the list |
| 11 Command palette | **partial — FAIL on the command rows** | ⌘K opens ✓. Selecting **New item** does nothing — finding 5. The row's own four remaining clauses (Esc, ↑/↓, Enter copies, ⇧Enter opens) are still unwalked |

Row 3 failing on the first surface walked is the argument for the whole manual pass. Both causes
are plain in the source and neither was found by reading it in three phases, because both are
about the moment **between** two renders — the screen is correct in every static reading of it.

### The sitting — 2026-08-14

**Reported by the author: the walk was run and every row it reached passed**, including the three
re-walks the fixes of 2026-08-08 had re-opened (row 3 after D-68/D-69, row 6 after D-67, row 11
after D-66). Nineteen boxes are ticked above on that report. No finding came out of it, which
makes it the first pass here that returned none.

**What this record is.** The rows above are ticked from the author's report, in the same way rows
7, 8 and 9 were on 2026-08-08 — a manual walk has no other evidence, and inventing per-row
observations nobody wrote down would make this document worth less than the memory it came from.
So the "what was observed" column is not back-filled for the sixteen new rows. The report is the
evidence and it is dated.

**The three rows that could not be ticked on the 14th closed on the 15th**, each in its own way,
and the way matters more than the tick:

| Row | How it closed |
|---|---|
| 12 New-item dialog | **Re-worded, not conceded.** The clause "the generator opens from inside it and returns focus" described a dialog nobody built; the row now says what exists — *Generate* fills the password field in place, reveals it, and leaves focus on it. **D-73**, the author's call, taken in the open rather than by quietly deleting a clause that could not pass |
| 20 Profile popover | **Walked on the rebuilt binary and passed**, including the clause today's fix created: one Tab leaves the menu instead of stepping through *Lock vault* into what sits behind it. This is finding 10 confirmed by a person rather than by a commit |
| 21 Edit-profile dialog | **Walked and passed on both open paths** — from the popover row, which is gone by the time the dialog closes, and from the Settings card's button, which is not. That is `Dialog.svelte`'s `isConnected` guard exercised on the path it was written for, which no other row reaches |

The record below is what was written when they were still open, kept because the reason a row was
held is worth more later than the tick that replaced it.

**Three rows were not ticked, and none of the three was a doubt about the walk.**

| Row | Why it is held | What closes it |
|---|---|---|
| 12 New-item dialog | Its clause *"the generator opens from inside it and returns focus"* describes a dialog that does not exist — `NewItemDialog.svelte:124` calls `generatePassword()` and fills the field in place. A row cannot pass a clause about a surface nobody built, and it cannot be re-worded here: that is the author's call, the same shape as D-64 and D-67 | Decide: build the nested dialog, or re-word the clause to the fill-in-place behaviour. Then walk it |
| 20 Profile popover | The surface is **not in any binary this machine has**. `target/release/trustvault` was built 2026-08-08 12:11; D-70's profile landed 18:13 the same day, and the roving-tab-stop fix (finding 10) on 2026-08-14 | Rebuild, then walk both rows. Five minutes, not a sitting |
| 21 Edit-profile dialog | Same build gap, plus it is the row that exercises `Dialog.svelte`'s `isConnected` guard on the path where the opener is **gone** by the time the dialog closes — the one thing here no other row covers | As above |

If the walk ran against a binary built somewhere other than `target/release/`, rows 20 and 21 tick
with a note saying where — the check is that the build carried the surface, not where it sat.

## Result

| | |
|---|---|
| S-08 target | 100 % of surfaces operable with no pointer |
| Global rules | **7 of 7**. Six by machine on every build — `npm run a11y`, **120 surface-audits, no findings** on 2026-08-14 — three audits over twenty scenarios in both themes. `taborder` joined `focus` and `contrast` that day and returned finding 10 on its first sweep; the number above is the sweep after it was fixed. The seventh rule is the manual pass below |
| Surfaces | **22 of 22 — S-08 is met.** Nineteen on the author's walk of 2026-08-14, including the three re-walks D-66, D-67 and D-68/D-69 had re-opened; the last three on 2026-08-15 — row 12 re-worded (**D-73**), rows 20 and 21 walked on a rebuilt binary carrying D-70's profile. No finding came out of either sitting |
| Date | 2026-08-07 (global rules), extended 2026-08-14 (tab order, and it is in CI); manual pass opened 2026-08-08, walked 2026-08-14, **completed 2026-08-15** |

**The total is 22 boxes**, and the number above is corrected rather than carried: the rows are
numbered 1–21, row 8a is a box alongside row 8, so the count is 21 + 1. Every "0 of 19" written
before 2026-08-08 was counting the highest row number instead of the rows, and the "of 20" written
earlier that day predates rows 20 and 21 (D-70). It changes no work and no row's wording. It is
corrected here because the gate line is the one that has to survive it, and that line was already
re-worded to need **a list rather than a total** — this is now the third time this table's count
has been wrong in the direction of a total, which is the argument for the list.
