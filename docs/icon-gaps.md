# Icon gaps

D-28 replaced Lucide with the 30-glyph set the design shipped (`src/lib/icons/ui/`). That set has
**no upstream**: a glyph it lacks has to be drawn or the control has to be re-worded. This file
records every gap the surfaces have hit and which of the two answers was taken, so the same
substitution is not re-argued each time somebody meets it.

`MASTER.md` §8's stronger clause is what makes this a list rather than a shrug: **one set, no
mixing.** Borrowing a single glyph from Lucide to fill a gap would break §8 in a way that is
invisible in review and permanent in the codebase.

## Resolved by re-wording

| Surface | Wanted | Used | Why it is acceptable |
|---------|--------|------|----------------------|
| Phase 0 workbench theme toggle | `palette` | `refresh` | The control cycles themes, so the glyph names the **action** rather than the subject. Weaker than a palette, not wrong. |

## Resolved by substitution

A substitution is a glyph that means something adjacent. Each row states what is lost, because
"close enough" is a judgement someone should be able to disagree with later.

| Surface | Wanted | Used | What is lost |
|---------|--------|------|--------------|
| Item list, detail pane | `id-card` / `passport` for `identity` | `user` | `user` reads as "a person", not "documents about a person". A vault holding a passport and a driving licence shows the same glyph as a login's username field. Worth drawing when the identity type gets its own surface in Phase 3. |
| Item list, detail pane | `key-round` or similar for `ssh_key` | `terminal` | `ssh_key` and `api_key` now share `terminal`, so the two are indistinguishable in the list. This is the **worst gap in the set** — two of seven item types collapse into one glyph. `key` is already taken by `login`. |
| Sidebar Watchtower entry | a distinct "shield with count" | `shield` | None materially. `shield-check` is reserved for the *strong* status, and using it for the nav entry would make a nav item look like a verdict. |
| Status chip, `reused` | `copy-check` or a duplicate-document glyph | `copy` | `copy` also means the copy-to-clipboard action elsewhere in the same window. The chip carries the word "Reused" beside it, which is what keeps it unambiguous — and is the reason §2 requires the word. |

## Still open

| Surface | Wanted | Current state |
|---------|--------|---------------|
| Detail pane, TOTP | a countdown ring | Phase 3. The design draws a ring, not a glyph, so this may be a component rather than an icon. |
| Command palette (⌘K) | `command` | Phase 3. No glyph in the set; the palette may not need one, since it is summoned by a keystroke rather than a button. |

## The rule for adding one

Draw it into `src/lib/icons/ui/` on a **24px viewBox at 1.6px stroke** — the shipped set's
geometry, not §8's nominal 16px/1.5px, because mixing two geometries inside one set is the same
mistake as mixing two sets. Then add the name to `NAMES` in `src/lib/icons/Icon.svelte`; the dev
server fails on the edit that causes drift rather than at the moment someone opens the one screen
using the missing glyph.
