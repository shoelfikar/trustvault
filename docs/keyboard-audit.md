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

- [ ] **`:focus-visible` on every interactive element**, distinct from the selection ring — a
      selected list row and a focused one must not look the same, or Tab is untrackable inside the
      list. `MASTER.md` §7 and §10.
- [ ] **Tab order follows visual order** on every surface. No positive `tabindex` anywhere in the
      codebase; a positive value reorders the whole document, not the component it appears in.
- [ ] **No keyboard trap.** From any focused element, Tab and Shift+Tab eventually leave — except
      inside a modal, where the trap is the point and Esc is the exit.
- [ ] **Every dialog traps focus, restores it on close**, and closes on Esc — the three together,
      because restoring focus to `document.body` passes the first two and strands the user at the
      top of the page.
- [ ] **Nothing is reachable only by hover.** Row actions that appear on hover must also appear on
      focus; this is the single most likely finding in the whole audit, and the detail pane's
      toolbar and the item list's row actions are where to look first.
- [ ] **No handler is bound to a bare printable key** outside a text field. `/` to focus search is
      the usual offender: it eats the character in every field that forgot to stop propagation.
- [ ] **`prefers-reduced-motion` honoured** — `MASTER.md` §10. Listed here because a focus ring
      that animates into place is a focus ring you cannot follow.

## Surfaces

Grouped as the design groups them. The exit gate says **15 surfaces**; the rows below are
enumerated from `src/lib/` rather than from that number, and **reconciling the two is owed at the
gate** — if the count disagrees, the source is right about what exists and the gate line needs a
list rather than a total. Recorded now so the disagreement is found at the audit and not argued
about on the last day.

### Before unlock

| # | Surface | Must be operable by | Done |
|---|---------|--------------------|------|
| 1 | Onboarding step 1 — name & location | Tab through name and path, Enter advances | [ ] |
| 2 | Onboarding step 2 — master password | Type, Tab to confirm, strength meter reachable but not focus-stealing, Enter advances only when valid | [ ] |
| 3 | Onboarding step 3 — recovery kit | Copy and Save buttons focusable; the acknowledgement checkbox toggles with Space; Enter finishes | [ ] |
| 4 | Lock screen | Password field focused **on mount**, Enter unlocks, the recovery link is Tab-reachable | [ ] |
| 5 | Recovery dialog, both steps | Esc closes from either step; Enter advances; the reissued kit's copy button is reachable | [ ] |

Row 4's "focused on mount" is the one autofocus in the application that is unambiguously right: the
lock screen exists to receive a password and has one field. Everywhere else, autofocus is a finding.

### The shell

| # | Surface | Must be operable by | Done |
|---|---------|--------------------|------|
| 6 | Sidebar — views, tags, vault switcher trigger | Tab in, ↑/↓ between entries, Enter selects | [ ] |
| 7 | Item list | ↑/↓ move selection, Home/End jump, Enter opens in the detail pane, type-ahead is **not** implemented (⌘K is the search) | [ ] |
| 8 | Detail pane — fields and toolbar | Tab through fields; per-field Reveal and Copy reachable **without hover**; Edit and Delete in the toolbar | [ ] |
| 8a | Detail pane — one-time code row | The **Copy code** button is a Tab stop like every other copy in the box, and the remaining seconds are readable as text rather than only as the arc — a countdown drawn in colour and geometry alone is unreadable to the people this audit is for | [ ] |
| 9 | Empty states — every list | The primary action is focusable and is the first stop after the list itself | [ ] |
| 10 | Titlebar / toolbar chrome | Search trigger, New item, and the lock button all Tab-reachable | [ ] |

Row 7 says what is **not** built as well as what is: type-ahead in a list competes with ⌘K for the
same keystrokes and would swallow the first letter of every shortcut a later phase adds.

### Overlays

| # | Surface | Must be operable by | Done |
|---|---------|--------------------|------|
| 11 | Command palette | ⌘K/Ctrl+K opens from anywhere, Esc closes, ↑/↓ navigate, Enter copies the password, ⇧Enter opens the item — R-16 | [ ] |
| 12 | New-item dialog | Type chips selectable with ←/→ and Space; every type's fields Tab-reachable; the generator opens from inside it and returns focus; the **New tag** field takes Enter to add a tag and does **not** submit the item; the **2FA switch** takes Space and Tab from it lands in the seed field it just revealed | [ ] |
| 13 | Delete confirmation | Focus lands on **Cancel**, not Delete; Esc cancels; Enter activates whatever is focused and nothing else; for a **vault**, the confirm field is reachable and the Delete button stays disabled until the typed name matches — R-18 | [ ] |
| 14 | Generator dialog | Length slider on ←/→ (and Home/End), the four set toggles on Space, Regenerate and Copy reachable — R-15 | [ ] |
| 15 | Vault switcher | ↑/↓ between vaults **including the open one**, Enter switches, Tab within a row reaches **Leave**, Esc closes — R-22 | [ ] |
| 16 | Settings | Every control reachable in visual order; the segmented controls on ←/→; **UI scale changes do not move focus**; the **Start at login** toggle takes Space, and when the platform refuses the write the toggle returns to its old position with the reason announced — R-21 | [ ] |
| 17 | Watchtower | The findings list is a list: ↑/↓ and Enter to the offending item | [ ] |
| 18 | Edit-item dialog | Every field row Tab-reachable in display order; **Replace** on a secret row reachable without hover, and Tab from it lands in the input it just opened; Remove and Add field reachable; the **New tag** field takes Enter as "add this tag" — R-17 | [ ] |
| 19 | Import dialog | Esc closes from the intro **and** from the report; Choose file… reachable and the **native picker takes over from there**, so the keyboard path leaves the app and must come back to a focused dialog; Import stays disabled until a preview is on screen; the report's refusal list scrolls with the keyboard alone, not only with a wheel — R-29 | [ ] |

Rows 18 and 19 were added 2026-08-06 with the surfaces themselves, and they are why this table's
count is now **19 against the gate's 15**: neither the edit dialog nor the import dialog is drawn
in the prototype, so both exist because a requirement needs one and not because the design has a
picture of it. That widens the reconciliation this section already owed rather than changing its
shape — the source is right about what exists, and the gate line needs a list rather than a total.

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

_None yet — the audit has not been run. It is run against the surfaces as they are wired, not once
at the end._

## Result

| | |
|---|---|
| S-08 target | 100 % of surfaces operable with no pointer |
| Measured | — not yet run |
| Date | — |
