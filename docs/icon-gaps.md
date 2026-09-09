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
| Item list, detail pane | `id-card` / `passport` for `identity` | `user` | `user` reads as "a person", not "documents about a person". A vault holding a passport and a driving licence shows the same glyph as a login's username field. Worth drawing when the identity type gets its own surface in Phase 3. **Still not drawn 2026-08-06, when all seven types became creatable**: unlike the row below it, `user` collides with no other *type* glyph, so what it costs is meaning and not distinguishability. Re-read at the §10 sweep. |
| ~~Item list, detail pane~~ | ~~`key-round` or similar for `ssh_key`~~ | ~~`terminal`~~ | **Closed 2026-08-06 by drawing, not by substitution — see below.** It read: `ssh_key` and `api_key` share `terminal`, so two of seven item types collapse into one glyph, the worst gap in the set. |
| Sidebar Watchtower entry | a distinct "shield with count" | `shield` | None materially. `shield-check` is reserved for the *strong* status, and using it for the nav entry would make a nav item look like a verdict. |
| Status chip, `reused` | `copy-check` or a duplicate-document glyph | `copy` | `copy` also means the copy-to-clipboard action elsewhere in the same window. The chip carries the word "Reused" beside it, which is what keeps it unambiguous — and is the reason §2 requires the word. |

## Resolved by drawing

| Surface | Glyph | Drawn for | What it is |
|---------|-------|-----------|------------|
| Item list, detail pane, palette | `code` | `api_key` | 2026-08-06. Left and right chevrons around a slash — `< / >`. |

The row above was written as *two* missing glyphs and closed with **one**, which is worth the
paragraph because the reasoning is not the obvious one. The collision looks like `ssh_key` needing a
glyph of its own, and the fix went the other way: `MASTER.md` §8 **assigns `terminal` to the ssh
key** by name, so `terminal` was never the substitute there — it is that type's glyph. What §8 does
is list six of the seven item types and omit `api_key`, and a type with nothing assigned to it is
how the two came to share one. Drawing a second key-shaped glyph would have left the real gap open
and put three key silhouettes in one list.

`code` names the world the credential belongs to, which is the pattern §8 already uses when it gives
the ssh key a `terminal` rather than a key. What is lost is that it does not say *key* at all: at
16px `< / >` reads "code", and it is the item's title beside it that says which credential. The
alternative that would have said key — a second key with a squared bow — loses more, because two
key silhouettes at 16px are told apart by their bow and nobody scanning a list looks at the bow.

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
