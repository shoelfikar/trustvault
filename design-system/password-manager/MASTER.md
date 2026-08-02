# Design System Master File — Password Manager (Tauri + Rust)

> **LOGIC:** When building a specific screen, first check `design-system/password-manager/pages/[page-name].md`.
> If that file exists, its rules **override** this Master file. If not, follow the rules below strictly.

**Project:** Password Manager (desktop, Tauri v2)
**Design Dials:** Variance 3/10 (centered/minimal) · Motion 2/10 (functional only) · Density 8/10 (dense/desktop)

### Provenance of these rules

Generated from the `ui-ux-pro-max` database, then **deliberately overridden** in three places. The
overrides are the whole point — the database defaults are the generic look we are avoiding:

| Dimension | DB default | Override | Reason |
|-----------|-----------|----------|--------|
| Style | "Exaggerated Minimalism" + "Trust & Authority" landing pattern | Instrument / control-panel UI | The DB matched a marketing site, not a resident desktop tool |
| Color | `#0F172A` slate + `#059669` emerald | Warm graphite + brass (see below) | Slate-900 + emerald is the single most over-generated palette in existence |
| Type | Inter (or all-JetBrains-Mono) | Geist Sans + Geist Mono | Inter-everywhere is the strongest "AI-generated" tell; all-mono is costume, not craft |

Everything else (accessibility, motion budget, anti-patterns, checklist) is kept from the DB.

---

## 1. Design intent

**The app is an instrument, not a landing page.** It lives in the dock, opens 15× a day, and is used
for 8 seconds at a time. Optimize for: scan speed, keyboard operation, zero ambiguity about lock
state. Never for: delight, storytelling, first-impression wow.

Reference feel: a machined tool — Things 3's restraint, Kagi's warmth, KeePassXC's information
density, none of the neon. The user must feel the app is *careful*.

### Explicit "not-AI" ban list

These are prohibited regardless of what any generator suggests:

- ❌ Indigo/violet (`#6366F1`, `#7C3AED`) or emerald (`#10B981`) as the brand accent
- ❌ Blue-cast slate neutrals (`#0F172A`, `#1E293B`) — they read cold and machine-picked
- ❌ Gradient buttons, gradient text, mesh/aurora blobs, glowing borders
- ❌ Glassmorphism / `backdrop-filter` as decoration (allowed only on the lock-screen scrim)
- ❌ Card grids with 16px+ radii and drop shadows for list data (use rows and hairlines)
- ❌ Emoji as icons, hero-scale typography, "✨ Sparkle" AI framing anywhere
- ❌ Purple-tinted shadows, `shadow-2xl`, `translateY(-2px)` hover lifts on rows

---

## 2. Color

Two themes, both first-class. Dark is the default (security tools live at night; OLED-friendly).
The neutral ramp is **warm** (hue ≈ 40°, saturation ≤ 6%) — this is the main thing separating it
from every slate-based UI.

### Dark theme (default)

| Role | Hex | CSS Variable | Use |
|------|-----|--------------|-----|
| Base / chrome | `#131211` | `--bg-base` | Titlebar, sidebar, window edge |
| Surface | `#1A1917` | `--bg-surface` | Main content pane |
| Raised | `#211F1D` | `--bg-raised` | Cards, dialogs, detail pane |
| Hover | `#2A2724` | `--bg-hover` | Row hover |
| Selected | `#33302B` | `--bg-selected` | Selected list row |
| Border | `#332F2B` | `--border` | Hairlines, dividers |
| Border strong | `#45403A` | `--border-strong` | Input outlines, focus track |
| Text primary | `#EDE9E3` | `--fg` | Titles, values (warm off-white, never `#FFF`) |
| Text secondary | `#A8A19A` | `--fg-muted` | Usernames, metadata |
| Text disabled | `#766F68` | `--fg-subtle` | Placeholders, empty states |
| **Accent (brass)** | `#C08A2E` | `--accent` | Focus ring, active nav, primary button, links |
| Accent hover | `#D6A044` | `--accent-hover` | |
| On accent | `#17150F` | `--on-accent` | Text on brass fill |
| Accent wash | `rgba(192,138,46,0.12)` | `--accent-wash` | Selected-row tint, chip background |

**Contrast:** accent on surface = **5.7:1**, on-accent on accent = **6.1:1**, `--fg` on surface =
**13.4:1**, `--fg-muted` on surface = **6.6:1**. All ≥ AA.

### Light theme

| Role | Hex | Variable |
|------|-----|----------|
| Base | `#EFEBE4` | `--bg-base` |
| Surface | `#F7F5F1` | `--bg-surface` |
| Raised | `#FFFFFF` | `--bg-raised` |
| Hover | `#EDE9E1` | `--bg-hover` |
| Selected | `#E6E0D5` | `--bg-selected` |
| Border | `#E0DACF` | `--border` |
| Border strong | `#C9C1B4` | `--border-strong` |
| Text primary | `#211F1C` | `--fg` |
| Text secondary | `#5F594F` | `--fg-muted` |
| Text disabled | `#8C857A` | `--fg-subtle` |
| **Accent (brass)** | `#8A5D0F` | `--accent` (darkened for 5.2:1 on light) |
| Accent hover | `#6F4A0B` | `--accent-hover` |
| On accent | `#FFFFFF` | `--on-accent` |
| Accent wash | `rgba(138,93,15,0.10)` | `--accent-wash` |

### Semantic — status only, never decoration

Brass is **brand/interaction only**. It must never mean "good" or "warning". Status colors are
reserved and desaturated so an all-red Watchtower screen doesn't look like a fire alarm:

| Meaning | Dark | Light | Variable |
|---------|------|-------|----------|
| Healthy / strong | `#4E9E63` | `#2F6B41` | `--ok` |
| Caution / reused / weak | `#C2643A` | `#8F4520` | `--warn` |
| Critical / breached / expired | `#CE5148` | `#A03229` | `--danger` |
| Info / neutral note | `#6E93B8` | `#3D617F` | `--info` |

**Mandatory:** status is *never* conveyed by color alone (DB anti-pattern `color-only-indicators`).
Every status carries an icon **and** a text label: `⛊ Breached` / `△ Reused` / `✓ Strong`, rendered
with Lucide glyphs, not those characters.

### Password-strength meter

Four discrete segments, not a rainbow gradient: `--fg-subtle` (empty) → `--danger` → `--warn` →
`--ok`. Always paired with the word Weak / Fair / Strong / Excellent and the zxcvbn crack-time
estimate in text.

---

## 3. Typography

**Bundle the fonts as local `.woff2` files. Do not `@import` from Google Fonts** — the app must
work offline and the Tauri CSP forbids remote font origins. Both faces are OFL.

| Role | Font | Notes |
|------|------|-------|
| UI / body | **Geist Sans** | Neutral grotesque, tighter and more mechanical than Inter, no "startup" smell |
| Data / secrets | **Geist Mono** | Passwords, TOTP codes, keys, hashes, hex, license keys |
| Fallback stack | `-apple-system, "Segoe UI Variable", ui-sans-serif` | Match host OS if bundling fails |

Acceptable alternates if you dislike Geist: **IBM Plex Sans + IBM Plex Mono** (more engineering-lab)
or **Instrument Sans + Martian Mono** (more editorial). Do not substitute Inter.

**Non-negotiable rule:** every rendered secret, TOTP code, and generated password uses the mono face
with `font-variant-numeric: tabular-nums` and disambiguated glyphs (0/O, 1/l/I must be distinct).
This is a correctness requirement for hand-transcription, not a style choice.

### Scale — desktop density, not web density

Desktop app convention is 13px body, not the web's 16px. This **knowingly deviates** from the DB rule
`typography-base-16`. Mitigations are mandatory: nothing below 11px, all contrast ≥ 4.5:1, and a
**UI Scale setting (Compact 92% / Default 100% / Large 115%)** that scales the whole `rem` root.

| Token | Size / line-height | Weight | Use |
|-------|-------------------|--------|-----|
| `--text-micro` | 11 / 14 | 500, `letter-spacing: .04em`, uppercase | Section labels, field captions |
| `--text-sm` | 12 / 16 | 400 | Metadata, timestamps, helper text |
| `--text-base` | 13 / 18 | 400 | List rows, field values, body |
| `--text-md` | 15 / 20 | 500 | Item titles, dialog body |
| `--text-lg` | 20 / 26 | 600, `letter-spacing: -.01em` | View headings ("All Items", "Watchtower") |
| `--text-xl` | 26 / 32 | 600, `letter-spacing: -.02em` | Lock screen only |

No display sizes. No `clamp(3rem, 10vw, 12rem)` anywhere — that was a landing-page rule.

---

## 4. Space, shape, elevation

Density 8/10 — 4px base grid.

| Token | Value | Use |
|-------|-------|-----|
| `--space-1` | 2px | Icon-to-label nudge |
| `--space-2` | 4px | Inline gaps |
| `--space-3` | 8px | Standard padding, row padding-y |
| `--space-4` | 12px | Row padding-x, field gaps |
| `--space-5` | 16px | Panel padding |
| `--space-6` | 24px | Section separation |
| `--space-7` | 32px | Dialog padding, empty-state breathing room |

**Fixed metrics:** sidebar 232px (resizable 180–320) · item list 300px (resizable 240–460) ·
titlebar 38px · list row 34px (compact 30 / large 40) · toolbar 40px · field row 30px.

**Radii — small and consistent:** `--radius-sm: 4px` (inputs, chips, buttons) ·
`--radius-md: 6px` (cards, dialogs, popovers) · `--radius-full` (avatars only). Nothing at 12–16px;
big radii are what make an app read as a web page pretending to be software.

**Elevation:** hairline borders do the work, shadows are for true overlays only.

```css
--shadow-popover: 0 4px 12px rgba(0,0,0,.28), 0 0 0 1px var(--border);
--shadow-dialog:  0 16px 48px rgba(0,0,0,.44), 0 0 0 1px var(--border);
```

Zero shadow on cards, rows, sidebar, or toolbar. No hover lift on rows — only `--bg-hover`.

**Touch-size deviation:** the DB requires 44×44px targets. That is a mobile rule; this is a
mouse-and-keyboard app, so interactive targets are **≥ 28px** in the pointer-fine case, with
`@media (pointer: coarse)` bumping rows and buttons back to 44px.

---

## 5. Motion — functional only

| Transition | Duration | Easing |
|------------|----------|--------|
| Hover / focus / color | 120ms | `ease-out` |
| Popover, dropdown, toast in | 160ms | `cubic-bezier(.2,.8,.2,1)` |
| Overlay/dialog out | 100ms | `ease-in` (exit faster than enter) |
| Lock/unlock scrim | 220ms opacity + 8px blur | `ease-out` |

**Forbidden:** scroll-reveal (the DB's GSAP preset does not apply — this app has no scroll
narrative), staggered list entrance, skeleton shimmer on a local vault that decrypts in <50ms
(show content, not theatre), animated page transitions, spring physics.

**The only expressive moment in the entire app** is the unlock transition: scrim blur clearing as
the vault opens. One moment, then the app gets out of the way. Everything honors
`prefers-reduced-motion: reduce` (drop to opacity-only, ≤80ms).

Copy-to-clipboard feedback: the button's icon swaps to a check for 1.2s + a countdown chip showing
clipboard auto-clear ("Clears in 12s"). No toast for a routine copy.

---

## 6. Layout

Three-pane, resizable, persisted across launches:

```
┌──────────────────────────────────────────────────────────────┐
│ ⌘ titlebar — vault name · search (⌘K/Ctrl+K) · lock button   │ 38px
├──────────┬───────────────────┬───────────────────────────────┤
│ Sidebar  │ Item list         │ Detail                        │
│ 232px    │ 300px             │ flex                          │
│          │                   │                               │
│ Vaults   │ ○ row · 34px      │  Title + type icon            │
│ ─────    │ ● selected        │  ────────────────────         │
│ All      │ ○ row             │  Field rows (label / value /  │
│ Favorites│ ○ row             │  copy / reveal)               │
│ Logins   │                   │  TOTP ring                    │
│ Cards    │                   │  ────────────────────         │
│ Notes    │                   │  History · Tags · Updated     │
│ ─────    │                   │                               │
│ Tags     │                   │                               │
│ ─────    │                   │                               │
│ Watchtwr │                   │                               │
└──────────┴───────────────────┴───────────────────────────────┘
```

- Below 900px width: detail becomes an overlay sheet over the list. No mobile breakpoints needed —
  this is a desktop-only app; the DB's 375px requirement does not apply. Minimum window 720×520.
- Titlebar: use native decorations for v1 (`decorations: true`) with `titleBarStyle: "Overlay"` on
  macOS only. Custom chrome on Windows/Linux costs more bugs than it buys.

---

## 7. Component rules

**List row:** 34px, `--space-4` padding-x, 20px type icon in `--fg-muted`, title at `--text-base`,
username at `--text-sm`/`--fg-muted` on the same line right-aligned. Status pips right-aligned.
Selected = `--bg-selected` + 2px `--accent` left bar. Focus (keyboard) is visually distinct from
selection: 1px `--accent` inset ring.

**Secret field:** value masked as `••••••••••` by default, in mono. Two icon buttons: reveal (eye)
and copy. Reveal is hold-to-show or a 10s auto-remask — never a permanent toggle that survives
navigation. Reveal on a masked field emits an audit-log entry.

**Buttons:** primary = brass fill, `--on-accent` text, 28px tall, 4px radius, 500 weight, no
gradient, no shadow. Secondary = transparent, 1px `--border-strong`. Destructive = `--danger`
text on transparent, filling only inside a confirm dialog.

**Destructive confirm:** deleting an item requires a dialog naming the item; permanently emptying
trash or deleting a vault requires typing the vault name. (DB rule `confirmation-dialogs`, High.)

**Empty states:** every list has one — line-art icon, one sentence, one primary action.
"No items in this vault yet" + `Add item`. Never a blank pane. (DB rule `empty-states`.)

**Command palette (⌘K):** the primary navigation surface. Fuzzy search across item titles, URLs,
usernames, tags; actions ("Lock vault", "Generate password", "New login"). Enter copies the
password by default; ⇧Enter opens the item. Arrow keys, Esc closes.

**Focus:** never remove outlines. `:focus-visible` = 2px `--accent` ring, 2px offset, on every
interactive element. Full keyboard operation is a hard requirement, not an accessibility extra —
this app is used one-handed while typing something else.

---

## 8. Icons

**Lucide**, 16px grid, 1.5px stroke, `--fg-muted` at rest / `--fg` on hover, `--accent` when active.
One set, no mixing, no emoji. Item types get distinct glyphs (`key` login, `credit-card` card,
`file-text` note, `user` identity, `wifi` wifi, `terminal` ssh key). If Lucide feels too common,
Phosphor (regular weight) is the approved alternate — pick one and never mix.

---

## 9. Anti-patterns (from DB, still binding)

- ❌ Excessive decoration ❌ Color-only indicators ❌ Emoji as icons ❌ Missing `cursor: pointer`
- ❌ Layout-shifting hovers ❌ Contrast below 4.5:1 ❌ Instant (0ms) state changes
- ❌ Invisible focus states ❌ Placeholder-as-label ❌ Errors surfaced only at form top

---

## 10. Pre-delivery checklist

- [ ] Fonts bundled locally as woff2; no network font request (verify with devtools offline)
- [ ] Both themes shipped; follows OS theme by default with a manual override
- [ ] All secrets render in mono with tabular numerals and disambiguated 0/O, 1/l
- [ ] Every status has icon + label, not color alone
- [ ] `:focus-visible` ring on every interactive element; full app reachable by keyboard alone
- [ ] Contrast audited in both themes at ≥ 4.5:1 body / 3:1 UI
- [ ] `prefers-reduced-motion` honored
- [ ] UI Scale setting works end to end (root `rem` scaling)
- [ ] No shadow on non-overlay surfaces; no radius > 6px outside avatars
- [ ] Destructive actions gated by a naming confirmation
- [ ] Every empty list has an empty state with an action
- [ ] Window state (size, pane widths) persisted and restored
