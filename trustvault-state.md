# TrustVault — State

Answers one question: where is this project now, and what happens next. It holds no requirements, no
calculations, and no task text — those live in `trustvault-requirements.md`, `docs/vault-format.md`,
and `phases/`.

Last updated: 2026-09-09, later (**Phase 4's remaining manual evidence is now recorded** — row
17's Watchtower re-walk and the two packet-capture runs had already been tested and were missing
from the written record. They are now marked complete in `docs/keyboard-audit.md`,
`phases/phase-4-watchtower.md` and `trustvault-requirements.md`, by user report on 2026-09-09.
Phase 4 is **24 of 24** and its exit gate is **7 of 7**; Phase 3 is still open only for the
seven-day drive. Earlier the same day: **a11y harness cleanup stopped masking the next Watchtower
walk** —
`scripts/a11y.mjs` now uses a fresh Firefox profile directory for each audit run, treats a failed
cleanup signal as cleanup failure rather than as an audit failure, and reports a browser timeout as
that rather than as "no tab stops", D-93. This was found by running the single Watchtower tab-order
audit: one run could not kill the launched Firefox process, the next invocation could not remove
the reused profile, and later runs in this desktop's Firefox headless path timed out before posting
a report. The capture harness self-test passes when local sockets are allowed: nine report checks
plus both sampler checks. No full a11y sweep, no Tauri release build and no heavy Rust test was run
in that sitting. Previous update: 2026-08-16, last (**the a11y job's 1 556 findings were a privacy notice** — the
failure yesterday's run left open is closed, and the cause was neither the audit's logic nor the
app. On a fresh profile Mozilla's own Firefox build shows the data-collection notice at startup,
whatever presents it takes **window activation** from the content, and from there
`document.hasFocus()` is `false`, `:focus` matches nothing, and every focus ring in the
application is invisible to `getComputedStyle` — while `contrast` and `taborder`, which never ask
who is focused, stay clean beside it. **The variable was the build and not the machine**: both
sides are 153.0.4, this desktop runs Ubuntu's snap which suppresses that notice and the runner
installs Mozilla's tarball which does not, so the audit had been measured for two days against
the one browser that hides the problem. Reproduced here by downloading the runner's build,
bisected to **one pref**, and verified by the full sweep against that build — **144
surface-audits, no findings**. `focusmanager.testmode`, the pref the symptom points at, was tried
first and measured to do nothing; it is not carried. The other half is **D-92**: `focus.js` now
measures **its own instrument** before the application, so this can never again arrive as 1 556
findings against the app. Nothing in TrustVault changed; at that moment the gate was untouched at
**2 of 7** and Phase 4 was still **23 of 24** — the remaining evidence was later recorded on
2026-09-09 by user report. Earlier: **the first real capture
run, and three defects in the thing built
to judge** — the harness met the running application and every fault was its own: the socket
sampler died on its first pass in all three runs (`set -e` inherited, `ss | grep` exiting 1 on the
quiet case, which is exactly what an `off` run is), the sweep reported twenty hits that were all
`password` inside `pwnedpasswords` (**D-91**), and ClientHellos were parsed only from attributed
packets so the destination check went quiet instead of failing. All three fixed, the sampler now
has a test that reproduces its own failure, and **the vacuity guard added hours earlier is what
stopped a `off` run that watched nothing from reporting "zero packets: pass"**. The record at the
time said both capture runs had to be taken again; the missing evidence was later supplied and
recorded on 2026-09-09 by user report. Then the branch was **pushed and CI ran on it for the first time** — run
31937787913, where **gate line 5 ticks** — S-07a at **62.5 ms** on a machine that is not this
desktop, against a 500 ms budget — and the a11y job **failed its first runner execution** with
100 % of focus checks reporting no-indicator, which is the audit rather than the app. **Gate line 1
ticks too**, on the author's own eyes rather than on a file: the app reported the fixture's three
`password` items breached at **52 372 427** against the live service. The gate was **2 of 7** at
that point; the five capture-related lines are now recorded complete as of 2026-09-09. Earlier the same day: the capture harness
itself — `scripts/capture.sh` with `capture-report.py` under it and `examples/audit-vault.rs` in
front of it, so the gate's evidence is a script's output rather than a person's grep. Three
decisions: **D-88**, attribution follows the process **tree**, because WebKitGTK does its
networking in a process the app's PID does not cover; **D-89**, TLS means the capture proves gate
line 2's negative and `hibp.rs` proves its positive, and both alternatives replace the thing under
test; **D-90**, the run is against the audit fixture, whose every string is published so the
forbidden-string list is generated rather than typed. The report script is a **CI step** and it
breaks its own capture eight ways to prove it can fail, and `probe` measured gate line 3's service
half the same day. Earlier: the breach check crossing IPC with **D-85**, **D-86** and **D-87**, the
HIBP client with **D-83** and **D-84**, the scan crossing IPC with **D-81**, the view reading a
real report with **D-82**)

## Overall progress

| | |
|---|---|
| Current phase | **4 — Watchtower, implementation complete**: entry check run 2026-08-15, **five of six** boxes — the sixth stays open for the life of the phase by D-74. **24/24** after the Watchtower row 17 re-walk and packet-capture evidence were recorded 2026-09-09 by user report. **Phase 3 is not closed** — 38/39 and gate 5 of 6, with the seven-day drive running underneath this phase |
| Phase document | `phases/phase-4-watchtower.md` — and `phases/phase-3-surfaces.md` stays open until the drive's day 7 |
| Phases passed | 3 of 6 |
| Last gate passed | **G-B′, on 2026-08-05** — Phase 2 closed at 26/26 with all five gate lines |
| Next gate | **Two were open at once, which is what D-74 bought.** Phase 3: 5 of 6, waiting only on the drive. Phase 4: **7 of 7**, with the last five capture-related lines recorded 2026-09-09 by user report from runs already performed |
| Status | on track — **the seven days are the only thing that cannot be shortened** before Phase 3 can close |

## Current phase

**Phase 2 is closed. Phase 3 is open** — entry check passed 2026-08-05, all six boxes, and the
notice marking the phase provisional is deleted. Work happens on `feature/phase-3-surfaces`, cut
from `development` *after* the entry check was recorded.

- **G-B′ passed 2026-08-05.** Four of its five lines carry measured evidence reproduced by CI (run
  30997297701); the fifth was run by hand on the app — vault created through onboarding, quit,
  relaunched, unlocked, and the relaunch landed on the **lock screen** rather than onboarding,
  which is D-40 working end to end. That line is an observation, not a measurement, and no CI job
  reproduces it
- Phase 2 closed at **26 of 26** tasks and 5 of 5 gate lines
- **Phase 3 is 33 of 39.** The kickoff document claimed 25 and listed 26 — the second phase document
  in a row with that slip, and the reason the rule says the checkboxes are authoritative and this
  number is a dated snapshot. Six tasks added with D-42, seven at the entry check
- **The whole "before any command is written" group is closed** (2026-08-05): the contract carries
  all fifteen Phase 3 commands, the item model is settled (D-43), `generate_password` is decided
  (D-44) along with two questions it dragged in (D-45, D-46), and `docs/keyboard-audit.md` exists
  as a checklist rather than a form
- **The first three Phase 3 commands are implemented** (2026-08-06): `add_item`, `update_item`,
  `delete_item`, with `FieldEdit` and `Item::apply_edits` under them. Twelve of fifteen remain.
  **Only one checkbox moved for it**, and that is not an accounting error: every task in the
  item-management group is worded for a *surface*, so the host half being done leaves them
  unticked. What the three now need is wiring — the New-item dialog, the detail pane's Edit, and
  the delete confirmation are all still drawn-and-disabled from D-36
- **The three are wired (2026-08-06, later the same day).** New-item saves, the detail pane's Edit
  opens a real dialog, and the delete confirmation deletes. Two of the group's boxes tick; the
  other three do not, and each says why in the phase document rather than being quietly counted
- **Wiring them found that no command with a two-word argument was reachable from the webview at
  all** — the whole of D-47, and the single most important thing in this session. Tauri v2 renames
  command arguments to camelCase by default, so the host wanted `itemId` while `src/lib/ipc.ts`
  sent `item_id`, which is what `docs/ipc-contract.md` prints and what that file converts to
  deliberately. `get_item`, `reveal_field` and `copy_field` had never worked from the frontend.
  **Nothing caught it for two phases**, and the reason is the shape of our own harness: the
  `_inner` split that lets `ipc_audit.rs` drive real command bodies skips exactly the argument
  decoding that was broken. It is the counter-example to Phase 2's "harnesses catch drift on their
  first run" — this one could not, by construction, and it took reading `strings` on the built
  binary to see it
- **The prototype has no edit surface** (D-48). Its detail-pane Edit button carries no handler,
  so the dialog built here is the first design decision in this project made without the
  prototype to follow. `MASTER.md` was the only authority, which is exactly the case `CLAUDE.md`
  says it wins
- **What Phase 3 inherits is chrome to wire, not chrome to build.** D-36 drew all 15 surfaces
  disabled, so the gate line "all 15 surfaces exist and are reachable" is half met on day one. It
  was re-worded at the entry check to mean **wired to a command**, and the phase's last task is a
  sweep for controls still disabled or wired to a stub — a stub is indistinguishable from a feature
  from the outside, which is exactly the risk D-36 named when it was taken
- **The 7-day daily-drive clock moved to the front of the phase.** It cannot start until the
  author's own passwords are in a vault, which needs `add_item` *and* the importer, so those are
  sequenced first. Left at the end it is not a test, it is seven days of waiting after the work
- **The Bitwarden importer landed 2026-08-06** (D-42's format, R-29's criterion), with
  `import_preview` and `import_commit` beside it and both `// planned` markers deleted in the same
  commit. Four of the import group's six boxes tick; the other two are the *surface*, which does
  not exist and is not a wiring job — the design predates D-42 and draws no import anywhere
- **Reading the schema instead of remembering it changed the work twice.** `CipherType` in
  `bitwarden/clients` has **eight** types, not the four every write-up quotes — 5 is an SSH key,
  which we *do* have a type for and a parser written from memory would have dropped. And an export
  can only ever produce **five of our seven kinds**, because Bitwarden has no API-key and no Wi-Fi
  type: that is now a test rather than a comment, because the tempting fix is a heuristic on the
  title and a heuristic is invisible once merged. It also means the gate's own R-29 line cannot be
  met as written — raised as an open question rather than edited here
- **The test that carries R-29 is the generic one.**
  `nothing_in_the_export_is_dropped_in_silence` walks the fixture and demands that every value in
  it is either in the vault or named in a refusal, which is the acceptance criterion as a rule
  instead of as a list. Verified non-vacuous the way the session harness was: one mapping deleted
  on purpose, and it fails naming `items2.card.brand = Visa`. Two decisions came out of writing
  it — **D-50** (a type we do not have becomes a note rather than a refusal) and **D-51** (an
  unknown *key* is a refusal, so a schema that grows cannot grow past us)
- **`ipc_audit.rs` now reads its own directory.** The `rename_all` check D-47 added iterated a
  hand-written list of command modules, which is the same shape as the bug it was written for: a
  new module would have been silently exempt. It now asserts the list covers every file in
  `src/commands/`, which is how `commands/import.rs` got checked on the day it was written
- **The generator group closed whole (2026-08-06, later the same day)** — all four boxes, host and
  surface together. `generate_password` is the **fourth sanctioned command** the contract has named
  since D-44 and `copy_generated` is beside it, so `ipc_audit.rs` now asserts a budget of four with
  the decision cited by number. What makes it worth more than four ticks: it is the first time
  D-37's two closed copy paths **open**, and the generator dialog is the first surface where a
  control drawn disabled by D-36 becomes a real command rather than a stub
- **The webview generator is deleted, not left beside the new one.** `src/lib/passwords.ts` is gone
  and all three surfaces that mint a password — the generator dialog, New item, Edit item — call the
  one command. That is D-44's own argument applied rather than quoted: two generators with one of
  them being "the real one" is a distinction that survives exactly as long as the person who
  remembers it. The retired file was not junk — `crypto.getRandomValues` with rejection sampling,
  deliberately never `Math.random` — which is what made deleting it the decision rather than the
  cleanup
- **Two things about the surface changed from the prototype and both are written down (D-52).** The
  footer said *Copy & autofill*, and autofill is a browser extension `trustvault-project.md` puts
  out of scope for v1 — the same class of false promise D-49 found in the delete copy, one screen
  over. And ambiguous-glyph exclusion is **not** a toggle: the prototype's own digit chip is
  labelled *2–9*, so the exclusion is a property of the sets rather than an option
- **The command palette group closed but for its measurement (2026-08-06, later still)** — four
  of five boxes, and `search_items` is the fifth Phase 3 command. Only one of the four is new
  work; the other three are the surface D-36 drew, read against R-16 and found to do what it
  says. What was new is the half nobody could see: the palette had been filtering a **client-side
  copy** on `title` and `tags` — two of R-16's four haystacks, and the exact shape D-46 rejected
  in advance. The matching now runs in `crates/trustvault-core/src/search.rs` and what comes back
  is the elided summary the list already gets, with **no record of what matched**
- **The rule about what is searched was falsified within the hour — the fourth time in four
  phases**, and by our own harness this time. Written as R-16's words (non-secret fields of kind
  `username`, `url`, `email`), it failed on the IPC fixture immediately: `Item::set_field` guesses
  `kind` from `secret`, so a username stored through it is a `text`. `kind` is how a field renders
  and is only as accurate as whoever wrote it; `secret` is what the *user* declared. The gate is
  now `secret` alone (**D-53**), which searches more than the requirement asks and cannot be
  quietly wrong — the failure it avoids is "why does this item not come up when I type its account
  name?", which no user can diagnose and no test would have caught
- **A palette that matched secret values would be an oracle, and that is now a test rather than a
  care.** Typing a guessed password into ⌘K would have the vault confirm it through the ranking:
  nothing revealed, nothing crossing IPC, no audit entry, and the answer on screen.
  `ipc_audit.rs` asserts a query equal to a stored password returns nothing, and the core asserts
  it for every field kind
- **The reference vault named in the requirements did not exist.** S-02, S-04, S-07 and R-11 have
  all said "1 000 items, 4 fields each, generated by `trustvault-core`'s `benchfixture` helper"
  since kickoff, and nothing implemented it — a requirement that reads as satisfied because it
  names a file. Written now, behind a feature so it stays out of the shipped library, and derived
  from the item index with **no RNG at all**: R-06's CI grep forbids a seedable one in this crate,
  and a hand-rolled generator written to pass the grep would be the rule broken with the evidence
  removed
- **S-04's host half is measured: 0.85 ms p95** (median 0.44, max 1.65; 34 000 samples over six
  queries and every prefix of each, `cargo bench --bench search`). That is 1.7 % of the 50 ms
  budget, so the criterion now rests on the IPC hop and the render rather than on the matching.
  The gate line stays unticked because the end-to-end number needs the running app — the palette
  carries the `performance.measure` marks S-04 names, and nobody has read them yet
- **A tag can now be created, not only chosen** — the cheapest thing on the list and one of the
  more embarrassing to have shipped without: the chips were the tags already in the vault, so a
  fresh vault offered none and no first item could ever be tagged. Both dialogs also draw the Tags
  control unconditionally now; it was hidden exactly when it was needed
- **The TOTP group closed whole (2026-08-06, later still)** — all three boxes, core, both commands
  and both surfaces, with `totp_code` and `totp_preview` deleting their markers in the same commit.
  Five markers remain. Two things make it worth more than three ticks. It is the **first group
  where the prototype draws both surfaces**, the exact opposite of D-48 one group over: the design
  was the authority for pixels and `MASTER.md` only had to arbitrate what the prototype promises
  and v1 does not ship (**D-55**, the *Scan QR on screen* button — the third prototype promise
  removed in three days, which is a pattern rather than three incidents). And the RFC's 18 vectors
  passed on the **first run**, across all three algorithms, which has not happened before in this
  project — the reason is that this is the first time the specification came with its own numbers
- **The risk was the parser, not the generator, and it was visible before a line was written.**
  `import/bitwarden.rs` has stored **either** a bare base32 seed **or** a whole `otpauth://` URI
  since the importer landed, because that is what Bitwarden emits. A generator reading only the
  first would have computed codes from *the letters of a URL* for every imported item and reported
  success doing it. `otpauth://hotp/…` is **refused** rather than read as TOTP for the same reason
  one level up: HOTP counts logins, not seconds, so every code derived from one is wrong and looks
  right. Both are named tests. This is the first time in four phases that the thing which would
  have gone wrong was caught by **reading our own code first** rather than by a harness afterwards
- **`totp_code` is the first vault-class command deliberately left un-audited**, and the reasoning
  is arithmetic rather than principle: the pane refreshes it every step, so an entry per call
  writes 120 an hour with the pane open and evicts every genuine reveal from D-31's 1000-entry cap
  before lunch. An audit log that is mostly its own noise is worse than none, because it still
  looks complete. The *seed* being revealed still goes through `reveal_field`, which does record it
- **The preview went into the Edit dialog too, which the task did not ask for.** A seed can be
  replaced there, so a validator guarding one of the two ways in is one that gets reported as
  "sometimes it checks" — and the edited seed is the more dangerous of the two, because the item
  worked before the edit. Factored into `TotpPreview.svelte` rather than duplicated
- **`ipc_audit.rs` caught the new module on its first run again** — the directory check D-47 added
  failed because `commands/totp.rs` was not in its list. Second time that check has earned itself,
  and this time by construction rather than by luck. The seed now lives in the harness's **shared**
  fixture rather than in the TOTP test alone, so checks 2 and 4 assert against a vault holding one;
  contract check 7 is automated in both halves for the first time
- **The frontend's `ErrorKind` union was missing `not_importable`.** Found while adding
  `malformed_totp_secret` beside it: the host has returned that kind since the importer landed
  2026-08-06 and `asIpcError` would have narrowed it to `internal`. Nothing had called the import
  commands from the webview yet — there is no surface — so it had never been reachable, which is
  exactly the shape of drift the contract's own check 1 cannot see
- **The settings-and-vaults group closed but for one box (2026-08-06, later still again)** — six
  of its own, plus the item-management delete box that had been half done since the morning. It is
  the largest group in the phase and it leaves **one `// planned` marker** in the whole contract,
  `list_tags`, which is deliberately unimplemented and says so. What the group actually added is
  the five `Settings` fields the contract has specified since the phase opened and the four vault
  commands, so R-21, R-22, R-27 and R-18's second half all land together
- **`launch_at_login` is the only setting in the application that writes outside this process**,
  and therefore the only one whose save can fail — an XDG desktop entry, a `LaunchAgent` plist, a
  `reg.exe` value, hand-written per platform (**D-56**). The ordering in `set_settings` is the
  requirement: the OS write happens **before** anything is stored, so a platform that refuses
  leaves the stored value alone and the toggle snaps back to the truth about the machine. It is
  also reconciled against the OS at start-up, because a user who removed the entry through their
  desktop's own startup tool has said something the settings screen must not go on contradicting
- **Window geometry joins `last_vault_path` as host-owned, and its trap is the sharper of the
  two.** D-40's field needs a frontend ignorant of it to spring; this one needs nothing — the
  webview holds a `Settings` from when its screen opened, so **resize the window, then change any
  setting**, and a frontend faithfully echoing every field it knows about sends the dimensions
  from before the resize. `merge_incoming` keeps four fields now rather than one, with a test each
- **`MASTER.md` was the document falsified this time, which has not happened before.** Four groups
  running, `docs/ipc-contract.md` has been the thing written first and proved wrong within the
  hour; here it was the design system. §3 has said since kickoff that UI Scale "scales the whole
  `rem` root", and `tokens.css` has **no `rem` in it at all** — every size is
  `calc(<px> * var(--ui-scale))`. The outcome is identical *only because nothing in the codebase
  uses `rem`*, which is a condition and not a fact, so **D-57** records the divergence, both
  documents are corrected in place rather than quietly re-worded, and a CI grep keeps the
  condition true. The failure it prevents is the nasty kind: a setting that scales most of a
  screen reads as a layout bug rather than as a broken setting
- **`delete_vault`'s confirmation was written into the half no harness can drive**, which is D-47's
  finding arriving from the other direction. The check R-18 asks for went naturally into the
  `#[tauri::command]` wrapper — and the `_inner` split that lets `ipc_audit.rs` and
  `ipc_session.rs` drive real command bodies skips exactly that layer. The single check in this
  product whose **success** is irreversible would have been the one check no test had ever read.
  The whole sequence moved into `delete_vault_inner`, which is also what makes its three orderings
  testable: the name before the lock, the key zeroized before the bytes, the file before the
  bookkeeping. Verified non-vacuous by loosening the comparison to case-insensitive, which fails it
- **`display_name_for` serves both `list_vaults` and the deletion check, and that is correctness
  rather than tidiness.** The contract defines the confirmation as equal to the display name the
  list reports, so two implementations that drifted would make a vault undeletable through its own
  dialog — the user typing exactly what is on their screen and being told it does not match
- **`known_vaults` grows in one place**, `Inner::opened`, which replaced the three identical
  blocks in `create_vault_inner`, `unlock_inner` and `unlock_recovery_kit_inner`. Consolidating
  them was not tidying: the list has to be updated on **every** path that leaves a vault open, and
  a fourth such path is added by someone who copies the vault and the path and does not know there
  were bookkeeping lines to copy — after which the switcher is missing the vault on screen
- **The keyboard audit earned itself before the surface shipped.** Row 15 asks for ↑/↓ between
  vaults *including the open one*, and the open row was drawn `disabled` first — the obvious way
  to say "you are already here", and wrong for one reason: a disabled button cannot take focus, so
  the arrow key stops dead on the row the user is standing in and reads as the key having failed.
  It is `aria-disabled` instead. This is the checklist working the way its own preamble said it
  would, which is the first time it has been used as a thing to build against rather than read
- **The `io` message said "the vault file"**, which for a failed login-entry write is the same
  class of wrong copy D-49 found in the delete dialog: one sentence, read at the moment it
  matters, describing something that did not happen. Re-worded to name a file rather than *the*
  file. `confirmation_mismatch` was already in the contract's §4 and **missing from the frontend's
  `ErrorKind` union** — the third instance of that exact drift, and each time for the same reason,
  that nothing had yet been able to reach the kind
- **The item-management group closed whole (2026-08-06, later still again again)** — its last two
  boxes, and with them the first Phase 3 group to reach 7 of 7. Neither was new capability, which
  is why both were left this long, and both turned out to be about something the app was *saying*
  rather than something it was missing
- **Every list already had an empty state; two of them were false.** The item list drew one message
  in five panes, so standing in Favorites in a vault of fifty items it read *"No items in this
  vault yet"*, and standing in Trash it read *"Deleted items sit here for 30 days"* — **the exact
  sentence D-49 deleted from the delete dialog that same morning**, still on screen one pane over.
  D-49 fixed the copy it was reported against and nothing swept for its siblings, which is the
  lesson worth more than the fix: a false sentence found in one place is a *class*, and the sweep
  belongs in the same session as the fix. The state is per view now, and the action follows the
  same rule — *Add item* where adding fills the pane, *Show all items* where it cannot, because
  Trash and an unused tag are dead ends you leave rather than fill
- **The palette's "no results" was a line of grey text**, which makes a failed search and a failed
  *load* look the same. It is an `EmptyState` with the search glyph and a *New item* action, and
  the action is not the duplicate it appears to be: a query matching no item has also filtered the
  New item **command row** out of the list, so it is the only way to act on what was just typed
- **Two of the seven item types stopped being the same glyph, and it took one new glyph rather than
  two** (**D-58**). The gap reads as `ssh_key` needing one of its own; `MASTER.md` §8 **assigns
  `terminal` to the ssh key by name**, so it was never a substitute there. What §8 does is name six
  of the seven types and omit `api_key` — a type nobody assigned anything to is how two of them
  came to share one, for two phases. `code` (`< / >`) is drawn for `api_key` in the shipped set's
  geometry, and §8 is corrected in place for that *and* for still saying **Lucide**, which D-28
  replaced two phases ago — a falsified line nobody had gone back for, in the same section
- **The screenshot harness now shoots the empty states**, three new scenarios in both themes: an
  empty vault, Trash, and the palette with a query that matches nothing. That is the point of
  the tool being there — the copy in a pane the author never visits is exactly what rots, and
  Trash is empty **by construction** in v1, so nothing but a scenario would ever put it on screen
  again. It cost one change to the harness itself: a scenario may now replace the item fixture
  outright, because "the list is empty" is a different fixture and not a flag on the same one
- **The import surface landed and with it the first plugin in the application (2026-08-06, later
  still again again again again)** — one box, and the box is not what the session was about. What
  stood between Phase 3 and its longest pole was never a screen: it was that the app had **no way
  to choose a file**, which is a dependency decision, and **D-59** is it. `tauri-plugin-dialog`
  answers both of the two tasks that wanted one — R-29's import, and the switcher's *Open vault
  file…*, drawn-and-inert since D-36 for exactly this reason
- **The dependency was taken and the webview granted none of it**, which is the whole shape of the
  decision rather than a detail of it. The plugin is registered in Rust so `commands::picker` can
  open a dialog; `capabilities/default.json` is still `core:default` alone, so the plugin's own
  `open`/`save`/`message` are denied and the two argument-less host commands are the only doors.
  That claim is **one line of JSON** away from being false at any moment, so it is check 8 in the
  contract's §9 and a test, not a sentence. The option that had to be refused on principle rather
  than on cost was `<input type="file">`: it hands the **webview** the bytes, and for an import
  those bytes are another password manager's plaintext in the heap that cannot be wiped
- **A dependency's cost is not only its tree, and this one proved it twice.** `tauri-plugin-dialog`
  pulls `tauri-plugin-fs` in as a library dependency — it registers no fs commands, but the crate
  is in the tree now and the next `cargo audit` will see it. And its init script **replaces
  `window.alert` and `window.confirm`** in our page, with a `confirm` that returns a **promise** —
  so `if (confirm(…))` is true always, and a confirmation written the way every web tutorial writes
  it would confirm itself. Neither is in the README; both came from reading the crate
- **The check written for that failed on its first run**, which is the third time in Phase 3 a new
  harness check has earned itself immediately. `DeleteDialog.svelte` declared a local
  `async function confirm()` shadowing the global — safe today, and safe exactly until somebody
  moves the call, on the one dialog in this product whose **success** is irreversible. It is
  `remove` now, and the rule is the blunt one (the identifier may not appear in `src/` at all)
  because a rule with an edge is a rule someone argues their way past
- **The screenshot harness caught a layout bug before a human saw the screen.** `.lede` was a flex
  container so it could hold a status glyph, which made the `<strong>` inside the intro paragraph
  a **column of its own** — the sentence rendered as three fragments side by side. Three scenarios
  were added (intro, preview, done) and the harness needed one change to take them: the load-hold
  is **per scenario** now, because the import flow is the first drive with three clicks in it and
  the default 900 ms expired mid-sequence, producing a photograph of the previous step. A shot of
  the wrong state still looks like a shot
- **The refusal list is drawn in full, with no "and 12 more".** R-29 is the rule that nothing is
  dropped in silence, and a list that hides its tail is that rule broken one indirection out where
  it reads as restraint. The report shown after the import is the **commit's**, not the preview's,
  because §6.8 has `import_commit` re-read the file — leaving the preview up would report the
  losing side of a real race as fact
- **One empty state was falsified by this change and swept in the same session**, which is the
  lesson the empty-states pass wrote down one day earlier. The vault switcher's empty state carried
  a comment explaining that it had no action *because this build could not pick a file*. That
  stopped being true the moment D-59 landed, and nothing prompts you to go back for a state written
  around a limitation when the limitation goes
- Blocked: nothing
- **Phase 2's branch was merged ahead of its own gate**, and it is worth keeping in view rather
  than filing away. PR #1 merged 2026-08-05 17:47 with G-B′ at 4 of 5 and an **empty body**;
  `90-MOC/Git Workflow Standard.md` asks a phase PR to carry the gate evidence. Nothing was lost —
  `development` was buildable, CI was green on the branch first, and the evidence is in the phase
  document — but a merged branch is the thing that later makes "the gate passed" and "the branch
  landed" look like one event. Phase 3's PR carries its evidence in the body

- **The D-36 sweep closed the phase's oldest risk (2026-08-07)**, and what it found is that all
  four remaining disabled controls had gone stale the *same way*: the thing each one's `title`
  said it was waiting for had arrived, and nobody went back for the sentence. Two are removed as
  promises v1 does not keep (**D-61**) — the recovery kit has no file format to load, because
  R-07's kit is whatever the user's own print dialog wrote, and *Rename vault* has no requirement,
  no command and no prototype behind it. Two are wired: onboarding's *Change* (**D-60**, a save
  dialog and the third door on `tauri-plugin-dialog`) and the switcher's *New vault* (**D-62**)
- **The New vault button was hiding a hole rather than a blemish.** `vault_status` answers
  `no_vault` exactly once in a vault's life, so onboarding was unreachable ever after and a user
  with one vault **could never create a second** — R-22 is a `must`, and it was reachable only by
  somebody who already had another `.tvault` from elsewhere. Wiring it then found the trap under
  it: `create_vault` writes the file whole and the flow opens with the same default name every
  time, so keeping "Personal Vault" for both would have written the first vault over with the
  second. No confirmation, no undo, and **no key in memory to have warned with**. `path_in_use` is
  the first refusal in this product that protects a file the user is not looking at
- **The two `MASTER.md` §10 lines that had never been measured are measured (2026-08-07)**, and
  the reason they never had been is structural rather than neglect: the app needs a Tauri host to
  render anything, so every claim about how it *looks* has been a claim about its CSS since Phase
  0. `scripts/a11y.mjs` walks the same fake application the screenshot harness photographs — the
  stub host moved into `scripts/harness.mjs` so a shot and an audit are evidence about one product
- **895 contrast findings on the first run, and the token behind them is the one the design calls
  "text disabled".** `--fg-subtle` measured **3.0:1 on a hovered row** while carrying group
  labels, counts, metadata and every placeholder in the product — **69 use sites** of a colour
  §2 describes as disabled. What makes it worth more than a palette fix is *why* §2 never caught
  it: the four contrast figures that section has always carried were all true and **all four were
  about `--bg-surface`**. A token is read on five backgrounds. **D-63** moves five tokens and the
  tool re-checks them, so §2 has stopped being the record and become the summary
- **The focus audit found itself wrong twice before it found anything about the app.** It reported
  the one autofocused control on every screen as ringless, because it measured an element that
  already had focus. And it **passed the real failure** — the command palette's search field,
  which had cancelled the global ring and replaced it with nothing since Phase 2 — because
  `outline-offset` still changes when the outline is `none`: a ring that is not drawn, moving. A
  tool that reports "no findings" is the easiest thing in this repo to believe and the hardest to
  check, so both mistakes are named in its own source
- **Ticking a checklist is not paperwork, twice over.** Going through §10 line by line found
  `Toggle.svelte` breaking the radius rule with a hardcoded 10px track since **Phase 0**, and
  found `Dialog.svelte` trapping focus and closing on Esc since **Phase 2** while never giving
  focus *back* — so closing any overlay left the user at the top of the application. The second is
  precisely what the keyboard audit's rule 4 is worded against ("the three together"), and it
  survived two phases because the two visible parts worked
- **The reference vault exists as a file now**, which was the last thing standing between S-04 and
  its end-to-end number: `benchfixture` built it in memory, and both S-02 and S-04 are about the
  *running* application. `cargo run --release --example reference-vault …` writes one in two
  seconds and **reopens it before reporting success**, because this file exists to be opened
- **What is left in Phase 3 is three tasks and none of them can be done from a text editor.** The
  keyboard audit's nineteen surfaces with the pointer physically unplugged, S-04's end-to-end
  measurement, and R-29's gate wording, which is the author's call. The critical path has been the
  author's since the import surface landed and it still is

- **R-29's wording was answered 2026-08-08 and the answer was not the paperwork it looked like
  (D-64).** The author took the proposed re-wording — *all of Bitwarden's item types, producing the
  five of ours they map onto* — and acting on it meant asserting the fixture covered them, which it
  did not. It covered **six of the eight**: types 7 and 8, the driving licence and the passport
  that `import/bitwarden.rs` names in its own comment, were in no fixture and no test, so D-50's
  generic path had been exercised by the bank account alone for two days. **The proposal itself
  carried the wrong number** — it said "all seven of Bitwarden's", our count borrowed into a
  sentence about theirs — which is the finding worth more than the fix: a re-wording written on the
  day the problem is found comes out of the same memory that produced the wrong line, and nothing
  re-checks it until somebody tries to tick the box. Both types are in the fixture now, the count
  is `the_fixture_covers_every_bitwarden_item_type` rather than a sentence, and the driving licence
  carries a `folderId` so the folder-merge test finally runs over a generic-path item
- **`cargo fmt --check` was already failing on the branch when this session opened**, on
  `examples/reference-vault.rs` from the previous commit — the phase's own Phase 2 lesson ("`cargo
  fmt` belongs after the last file is written") repeating on the last file written. It would have
  failed CI on the phase PR rather than on anything anyone was looking at. Fixed here
- **Phase 3 is 37 of 39, and the gate is 2 of 6 lines.** What remains is one sitting with the app
  running and seven days of calendar: the keyboard audit's 19 surfaces with the pointer unplugged
  (which also ticks §10's twelfth box and S-08), S-04's end-to-end number, the functional gate line
  D-38 moved here from G-B′, and the 7-day drive. **Nothing left is a repo change**
- **"Nothing left is a repo change" was wrong by one, for the second day running (D-65).** The line
  above is about tasks; what it missed is that S-04's procedure never said **which build** the
  number comes from, and the two candidate answers are not equivalent. `tauri dev` is the only
  documented way to launch this app and the only build with a console — and the workspace manifest
  optimizes dependencies while deliberately leaving our own crates unoptimized, so the matching
  that costs 0.85 ms p95 in release costs **6.63 ms** there (`cargo bench --bench search --profile
  dev`). The sitting would have produced a number, 12 % of it spent by a build nobody ships, and
  **nothing afterwards could have told it from the real one** — the same shape as every other
  measurement this project has had to go back for. The `measure` feature is the fix: a release
  build with `tauri/devtools`, opened on launch, opt-in, and held opt-in by a CI check, because an
  inspector attached to a process holding decrypted secrets is not something a release ships
- **Preparing the sitting is now the whole of what the repo can contribute.** The measure build and
  the reference-vault file are produced ahead of it, so the author's hour is spent walking surfaces
  rather than waiting on `lto = true`

**Phase 4 is open and its first code exists — 4 of 21, 2026-08-15.** The contract came first (§6.9),
then both unmeasurable gate lines came back split rather than relaxed (D-77), then the local scan:
`crates/trustvault-core/src/watchtower.rs` carries zxcvbn scoring and reuse detection, with no
socket in its dependency tree.

- **The scoring was already written, in the wrong crate — D-78.** `score_password` has served
  onboarding's meter from `src-tauri` since Phase 2, and its own comment gives the reason it is in
  the host: 400 kB of dictionaries must not be parsed by the **webview** on every cold start. That
  argument survives the move untouched. What it does not answer is host-versus-core, and scoring
  every password means reading every password — from `src-tauri` that is a thousand plaintext values
  lifted across a crate boundary and dropped again. It is a pass-through now, and "Weak" has one
  definition rather than two that agree until one is edited
- **The weak threshold was set by two failing tests, not by a choice — D-79.** The first draft
  assumed the meter's "Weak" band, score ≤ 1, and two tests written from that assumption failed.
  Measured: `hunter2` scores 1 and dies in under a second, `Tr0ub4dour&3` scores **2** and falls in
  **31 minutes**. The Watchtower view has read *"Crackable in a matter of hours"* under its Weak
  group since D-36 drew it from the prototype — **the design's own copy was the evidence**, written
  months before the decision it settled
- **It left two screens calling one number two words**, and that is an open question rather than an
  edit: score 2 is *Fair* on the meter and *Weak* in Watchtower. `MASTER.md` §2 makes status wording
  binding, so it is not the kind of thing whoever noticed it gets to rename
- **Three shapes the report carries on purpose.** A clean field is the **absence of a row**, never a
  row saying `strong`. A field that is both weak and reused produces **two findings**, and
  `ItemStatus` is where the ranking happens, because the item list draws one pip and has to pick.
  An item storing one password in two of its own fields is **untidy, not reused** — reporting it
  would put a row on screen naming the item as sharing with itself
- **The grouping key is SHA-256 and never leaves the module.** Hashed rather than grouped on the
  value for a memory reason as much as a secrecy one: a map keyed on `String` copies every plaintext
  password into an allocation nothing zeroizes. It is still a secret, so it is private, unprintable,
  and never returned — `shared_with` names other **items**, which the list already carries
- **What is not done and is worded as if it were**: the crack-time box. The string exists in every
  finding; nothing renders it, because the view still draws from the status cache. The box stays
  unticked — the Phase 3 lesson about surface-worded tasks, applied rather than re-learned

**Then the two tasks the re-worded gate added — 6 of 21, 2026-08-15.** A fixture with something to
find, and the measurement that fills S-07a's blank.

- **`auditfixture.rs` is twenty-one items and every number in it was worked out by hand.** Reuse
  groups of 3, 2, 5 and 2; a password at every zxcvbn score; one item storing one value in two of
  its own fields; one item with no password at all. It is a written table, not a derivation — which
  is what `benchfixture.rs` is, and the reason that one has nothing for R-23 or R-24 to see
- **The scores are read from zxcvbn on every test run, not asserted once.** The point is what a
  dependency bump does: zxcvbn is a dictionary and a set of matchers, both of which move between
  releases, and a fixture that still claims a spread across five scores while holding three makes
  R-24's tests pass by having nothing in them. The test names the row that moved
- **One row exists to fail if `scan` stops passing context.** *Northwind Mail* stores
  `priya.raman.2024` and its username is `priya.raman`: **4 bare, 2 in context**. Verified by
  dropping the argument on purpose — two tests fail, one of them naming the row. Nothing else in
  the project would have noticed the D-12 amendment being undone
- **Every test passed on its first run, which is why they were all broken on purpose before being
  believed.** Phase 2's harness rule, applied to a fixture rather than to a harness
- **S-07a is 61–63 ms against a budget of 500 ms — D-80.** The budget is 8× the reading and every
  multiplier in the gap is written down rather than left as slack: ×1.9 for zxcvbn's expensive case
  (the same benchmark scans the audit vault at 0.118 ms a password against the reference vault's
  0.061 ms), ×2 for a machine that is not this desktop, and the rest is headroom to where a command
  that returns reads as a hang — the local scan reports no progress and **cannot**, because
  `watchtower-progress` is the breach half's
- **The benchmark fails rather than prints.** `kdf.rs` and `search.rs` print, correctly: neither can
  run anywhere but this desktop. This one can run anywhere, so it is a CI step in the `rust` job —
  which is what the gate line already claimed and what nothing in `.github/workflows/` actually did.
  S-07a is the only measured criterion here a runner can re-take
- **Two things the measurement found that nobody was looking for.** The budget is per **1 000
  items**: at 0.12 ms a password on the expensive side, a 10 000-item vault is over the line and
  needs the progress events the breach half already has. And a scan of the reference vault returns
  **ten weak findings** — the single-digit indices, whose `pw-{index}-xK9` is eight characters and
  scores 2 — so any later test asserting that vault scans clean asserts something false
- **The gate line for S-07a still does not tick**, and that is deliberate. A reading cannot fail a
  budget computed from it; what makes it a gate line is a run that could have failed, which needs
  the scan reachable through `watchtower_scan` and the benchmark re-run somewhere other than the
  machine that set it

## Gates

Three states only, and a gate the project passed through without actually running is **not ticked** —
it is marked `not run — retrofitted` with the risk carried written next to it.

- [x] **Phase 0** — window opens with the token specimen in both themes; all CI quality gates green. Passed 2026-08-02, run 30747639101
- [x] **Phase 1** — format spec + KAT vectors committed; fail-closed proven; ≥ 90 % coverage; zero unsafe. Passed 2026-08-03. All seven criteria have evidence; coverage measured at 99.21 % lines / 100 % functions
- [x] **G-B′** — IPC security boundary holds: no secret crosses IPC outside the sanctioned path.
      Passed 2026-08-05. Four of its five lines carry measured evidence reproduced by CI (run
      30997297701); the fifth, the functional line, was run by hand on the app because nothing
      else can run it
- [ ] **Phase 3** — all 15 surfaces reachable by keyboard alone; `MASTER.md` §10 ticked; 7 days daily-driven
- [x] **Phase 4** — breach detection verified; zero egress when opted out, proven by packet capture.
      Passed by already-run manual evidence recorded 2026-09-09 by user report
- [ ] **G-C** — S-01…S-11 filled in with measured results
- [ ] **Phase 5** — signed artifacts for all three targets, each installed and version-asserted in CI

**G-A (pin mapping locked) is dropped** — software-only project, no hardware track. **G-B is replaced
by G-B′**, because the equivalent "the physical thing behaves as drawn" moment here is the IPC
security boundary, not a bring-up. Both recorded as decision D-14.

## Decision log

Every decision with the alternatives rejected — what's needed later is the constraint that produced
the choice, not the choice.

| Date | Decision | Reason | Alternatives rejected |
|------|----------|--------|-----------------------|
| pre-kickoff | Tauri v2 + Rust for a desktop app | Predates this document set; recorded in `design-system/password-manager/MASTER.md` | Electron, native per-platform — **no prior-art survey was run for this choice**, so it is a risk carried, not a considered decision |
| 2026-08-02 | **D-02** Svelte 5 + TS + Vite for the frontend | Smallest bundle and fastest cold start, which is what a dock-resident tool opened 15×/day is judged on (S-01) | React 19 — heaviest runtime, though the prototype is already React so the port is now manual work we chose to pay for. SolidJS — React-like ergonomics but a smaller ecosystem. Leptos — one language end to end, rejected for WASM payload and total prototype rewrite |
| 2026-08-02 | **D-03** Offline only; no sync in v1 | The design contradicted itself — sidebar said "Synced 2 min ago", Settings said "Offline — this device only". Resolved to offline. Sync needs conflict resolution, a server, and key exchange | Building sync now; leaving the contradiction unresolved in the UI copy |
| 2026-08-02 | **D-04** Custom `.tvault` format, Argon2id + XChaCha20-Poly1305 | Matches the design's own artefacts (`.tvault` filenames, Argon2id onboarding copy, recovery kit) and leaves room for Watchtower and audit metadata | KeePass KDBX4 (`keepass-rs`) — battle-tested and gives users an exit path, but has no native slot for status/audit metadata and no recovery-kit concept. SQLCipher — real queries, but a C dependency and a poor fit for a single portable file. `age` — file encryption, not a vault format: no per-item crypto, no key rotation story |
| 2026-08-02 | **D-05** Tauri Stronghold plugin not used | It is a secrets store with its own format — adopting it means a second vault inside the vault, with two formats to version | Using Stronghold as the storage layer; using it only for the master key |
| 2026-08-02 | **D-06** XChaCha20-Poly1305, not AES-GCM | The 192-bit nonce is large enough to be drawn randomly with no counter discipline. Aegis's format docs reject counter-based nonces outright for exactly this failure mode | AES-GCM (96-bit nonce, needs a counter or a birthday-bound argument), AES-GCM-SIV (misuse-resistant but less common in Rust and slower) |
| 2026-08-02 | **D-07** Ship Linux, macOS, and Windows from v1 | Decided at kickoff so signing costs and CI matrix are phase-0 problems rather than release-day surprises | Linux-only first — cheaper, but defers the two hardest distribution problems to the end |
| 2026-08-02 | **D-08** Native installers, not a `curl \| sh` installer | `90-MOC/Release Installer Standard.md` governs single-binary CLI apps. A GUI app with a webview runtime cannot build to one binary. The standard's checksum and clean-environment smoke-test requirements are kept and moved into the Phase 5 gate | Following the standard literally; shipping archives only with no installer |
| 2026-08-02 | **D-09** No component library | shadcn-svelte and Skeleton default to radii, shadows, and densities that `MASTER.md` explicitly bans; every component would be fought rather than used | shadcn-svelte, Skeleton, Melt UI (kept as a possible headless-primitives fallback if the dialog/focus-trap work proves fiddly) |
| 2026-08-02 | **D-10** CBOR (`ciborium`) for the vault body | Tolerates unknown fields, so a vault written by a newer version round-trips through an older one without data loss (N-09) | `postcard` — smaller and faster but schema-rigid, an unknown field is a parse error. JSON — larger, and no clean binary field. `bincode` — same rigidity problem as postcard |
| 2026-08-02 | **D-11** `arboard` (1Password's fork) for clipboard | Needs the platform MIME hints (`x-kde-passwordManagerHint`) that thinner wrappers do not expose, and the crate documents the Wayland ownership caveat that bites password managers | `tauri-plugin-clipboard-manager` — simpler but no MIME hint control; `copypasta` — less maintained |
| 2026-08-02 | **D-12** `zxcvbn` crate for strength scoring | The design's crack-time strings ("takes ~8 centuries to crack") are zxcvbn's own output format, so using anything else means reimplementing its phrasing. **Amended 2026-08-04:** the example in the rejected-alternatives column is **wrong, and was never measured.** zxcvbn scores `Jakarta2019!` as **3 / Strong** with its default dictionaries — "Jakarta" is not in its English-centric frequency lists, so it does not catch the case this row claims it catches. The decision stands on the phrasing argument and on the cases it does catch (`password` scores 0, `hunter2` scores 1). What closes the real gap is passing context through the `inputs` parameter, which `score_password` now does with the vault name; `crates`-side test `a_local_word_is_only_caught_when_zxcvbn_is_told_about_it` pins both halves so the claim is not re-derived from this row and believed | Entropy-only scoring — cheap, and it reports `Jakarta2019!` as strong. So, it turns out, does zxcvbn without context; the difference between them is smaller than this row originally asserted |
| 2026-08-02 | **D-13** Six phases, none merged | The 'no plaintext across IPC' rule needs its own gate; folding it into a larger phase is how it becomes an aspiration | Merging 0 into 1; compressing to three phases |
| 2026-08-02 | **D-20** `src-tauri/icons/` stays committed although it is derivative | `90-MOC/Gitignore Standard.md`'s single test says ignore it — one command regenerates it. Two things override that. It is a shipped artifact, so building it would let a `tauri` CLI upgrade change the user-visible icon with no diff to review; and measured on the day, `tauri icon` is **not byte-deterministic** — the same input yields a different `icon.icns` each run, so ignoring it would add unreviewable churn to every CI run rather than removing 380 KB of noise. `scripts/make-icon.py` was added so `icon-source.png` is itself reproducible byte-for-byte, which is what the artifact was missing | Ignoring `src-tauri/icons/` and generating during the build (churn, and an unreviewed icon change on every CLI bump); leaving `icon-source.png` as a binary nobody could regenerate, which is the weakest possible reason to commit something |
| 2026-08-02 | **D-18** Workspace MSRV raised from 1.85 to 1.88 | 1.85 was picked as a conservative default and turned out to be actively harmful: it pinned `cargo update` to versions still carrying RUSTSEC-2026-0009 (`time`) and RUSTSEC-2026-0194/0195 (`quick-xml` via `plist`). The patched releases require 1.88. An MSRV below what the security patches need is an MSRV that blocks them | Staying on 1.85 and ignoring three real DoS advisories; `--ignore-rust-version` in CI, which fixes the lockfile while leaving the manifest lying about what the project needs |
| 2026-08-02 | **D-19** `.cargo/audit.toml` ignores 17 advisories individually, with reasons | The GTK3 binding crates, `glib`'s unsoundness, and the `unic-*`/`proc-macro-error` crates all arrive through Tauri's Linux backend and cannot be fixed from here. Leaving CI permanently red on them trains everyone to ignore CI, which costs more than the advisories do. Each entry names why it is unfixable and what retires it, and anything not listed still fails | A blanket `--ignore-warnings`, which would hide new findings too; leaving the job red, which makes the signal worthless; dropping the audit job entirely, which is how N-04 quietly dies |
| 2026-08-02 | **D-17** The repository is public: `github.com/shoelfikar/trustvault` | Two reasons. A password manager asking for trust should be auditable, and a public repository gets unlimited GitHub Actions minutes — which matters because CI builds Linux, macOS, and Windows on every push, and the macOS runner bills at a 10× multiplier on private repositories. Consequence carried: every commit message, the design system, and anything pushed by mistake are permanently public, so the pre-push secret scan becomes a habit rather than a one-off | Private — safer default and trivially flipped to public later, rejected for the Actions cost and because the audit argument only works if the code is actually visible |
| 2026-08-02 | **D-15** `@lucide/svelte`, not `lucide-svelte` | The package installed first emitted a deprecation notice on install: `lucide-svelte` is superseded by the scoped package. Swapped before the first commit rather than carrying a deprecated dependency into the history | Staying on `lucide-svelte`; Phosphor (the approved alternate in `MASTER.md` §8) — no reason to switch icon families, only packages |
| 2026-08-02 | **D-16** `scripts/dev.sh` strips the snap environment before launching | The editor on this machine is a snap, and its integrated terminal exports `SNAP_LIBRARY_PATH`, `LOCPATH`, `GTK_PATH`, and `GIO_MODULE_DIR` pointing into `/snap/core20/`. A natively-built binary started from that terminal loads the snap's libc and dies before `main()` with `undefined symbol: __libc_pthread_init`. The binary is fine; the environment is not | Telling the developer to always use an external terminal (works, but is a trap that will be rediscovered every few months); patching `LD_LIBRARY_PATH` only (insufficient — the GTK and GIO module paths poison it too) |
| 2026-08-02 | **D-21** RustCrypto for both primitives: `argon2` 0.5.3 and `chacha20poly1305` 0.11 | Pure Rust, so all three targets cross-compile with no C toolchain and no vendored build script — which is what makes the Phase 5 signing story tractable. `chacha20poly1305` carries an NCC Group audit (funded by MobileCoin, no significant findings) and documents constant-time execution. Both are the reference implementations of their algorithm in the Rust ecosystem | `ring` — no Argon2id and no XChaCha20 at all, so it cannot implement this format. `libsodium`/`sodiumoxide` — the C original, but a C dependency on three platforms plus an unmaintained binding. `orion` — pure Rust and pleasant, far less deployed and no audit. `rust-argon2` — Argon2 only, no AEAD, so two ecosystems instead of one |
| 2026-08-02 | **D-22** `argon2` **0.5.3**, not 0.6.0-rc.8 | 0.6 is a release candidate and its `kdf` feature is exactly what this crate wants — the raw KDF without the PHC-string layer. A Tier-1 crate whose failure mode is unrecoverable data loss does not ship a release candidate of its key derivation. Cost carried and written into `Cargo.toml`: the `alloc` feature drags in `password-hash`, `base64ct`, and `rand_core`, none of which this crate calls | 0.6.0-rc.8 — smaller tree, better API, unreleased. Staying on defaults — pulls the PHC parser as well |
| 2026-08-02 | **D-23** `getrandom` called directly; **no** `rand` crate anywhere in the core | R-06 forbids counter-derived nonces, and the reliable way to keep that true is to have nothing in the tree that *could* become a counter. `getrandom::fill` is the OS CSPRNG with no userspace state, no seeding, and no `SeedableRng` to reach for in a hurry. A CI grep enforces both halves: no seedable RNG in the crate, and `getrandom` called from exactly one file | `rand` — ergonomic, but ships `StdRng`, `SmallRng`, and `from_seed`, any of which satisfies a compiler in a moment of haste. `chacha20poly1305`'s `getrandom` feature — the same function reached through one more layer |
| 2026-08-02 | **D-24** `data-encoding` base32 for the recovery kit, RFC 4648 alphabet, no look-alike mapping | The alphabet excludes `0`, `1`, `8`, and `9`, which removes the `0`/`O` and `1`/`I`/`l` failures a printed, hand-typed secret is otherwise guaranteed to hit. A mistyped digit is **rejected**, not mapped: silently reading `0` as `O` would mean two different printed kits open the same vault | Crockford base32 — maps look-alikes by design, rejected for exactly that reason. Hex — 30 characters for the same entropy and `0`/`O` back again. BIP-39 words — friendlier to read, but a 2048-word list and a language question in a v1 that has neither |
| 2026-08-02 | **D-25** The key wraps authenticate the **parameter block** (header bytes 0..20), not the salts | Written the other way first — both salts as shared associated data for both wraps — and the test suite failed within the hour with a precise symptom: changing the master password invalidated the *recovery* wrap. It cannot be re-wrapped at that moment, because the user is not holding their recovery code. Nothing is lost by narrowing it: a salt is already a KDF input, so editing it derives a wrong key, and both salts are still covered by the body tag whose AAD is the whole header | Both salts as shared AAD — the original design, which makes password change and kit reissue mutually destructive. Re-wrapping both on any change — impossible without both credentials in hand |
| 2026-08-02 | **D-26** Item `status` is stored in the vault as a **cache** of the last Watchtower scan | The design draws status pips in the item list, and re-scanning 1 000 items on every open to draw them would blow S-01. Stored with the honest label: stale by definition until the next scan, and nothing may make a security decision from it | Computing on open — correct but too slow, and it makes the list depend on the network when breach checking is on. Not storing it — the pips then appear only after a manual scan, which is not what the design shows |
| 2026-08-02 | **D-27** History of overwritten field values lives inside the sealed body | Losing a password to a mistyped edit is unrecoverable otherwise, and the alternative places are all worse | A separate history file — a second thing to encrypt, key, and keep in step. No history — cheapest, and the support case it creates has no answer |
| 2026-08-03 | **D-28** The design's own 30-glyph set replaces Lucide; `@lucide/svelte` removed | Directed by the author: the design shipped `icons/ui/`, drawn to match the mark, and those are the glyphs the prototype's surfaces were composed against. **This departs from `MASTER.md` §8**, which names Lucide (or Phosphor) and a 1.5px stroke; the set is custom at 1.6px on a 24px viewBox. §8's stronger clause — *one set, no mixing* — is what forced the dependency out rather than leaving it beside the new set, which would have broken §8 twice over instead of once. Cost carried: no upstream, so a glyph the set lacks has to be drawn, and the first one already surfaced — there is no `palette`, so the workbench theme toggle uses `refresh` | Keeping Lucide and ignoring the shipped set — rejected by the author's instruction. Keeping both and reaching for Lucide only where the set has a gap — the exact mixing §8 bans, and the gaps would never then be filled. Redrawing the shipped set onto Lucide's 16px/1.5px geometry — a week of work to end up with the icons we already have |
| 2026-08-03 | **D-29** The packaged app icon is rasterized from `src/lib/icons/brand/app-icon-macos-1024.svg`; `scripts/make-icon.py` retired | Directed by the author: use the icon the design shipped, stop maintaining a second one. `make-icon.py` existed only to make `icon-source.png` reproducible (D-20) — it re-drew the mark in Pillow, so the repo carried two independent definitions of the same artwork that could silently diverge, and the Python one was the copy. `tauri icon` takes an SVG directly, so the design file is now the single source and the intermediate PNG is deleted. **This supersedes D-20's reproducibility clause only**; the rest of D-20 stands unchanged — `src-tauri/icons/` stays committed, for the same two reasons (a shipped artifact whose change must be reviewable, and `tauri icon` still not being byte-deterministic) | Keeping `make-icon.py` and hand-syncing it against the SVG — two definitions of one mark, and the drift is invisible until someone compares renders. Committing the brand SVG as `icon-source.svg` in `src-tauri/` — a third copy of a file already in the repo. Generating the icon set during the build — rejected under D-20 already, unreviewable churn |
| 2026-08-03 | **D-30** `scripts/dev.sh` builds the child environment from an **allowlist** (`env -i` plus named variables), not by unsetting known snap variables. Supersedes D-16's mechanism; D-16 stands as the reason the script exists | D-16's blacklist was incomplete and cannot be completed: `XDG_DATA_DIRS`, `XDG_DATA_HOME`, and `GST_PLUGIN_SYSTEM_PATH` still pointed into `/snap/code/*` after it ran, and every snap revision may add another name. The measured root cause, which D-16 recorded only as "loads the snap's libc": the snap's GTK modules carry **`RPATH`** — not `RUNPATH` — of `/snap/core20/current/lib/x86_64-linux-gnu` (`readelf -d` on `libpixbufloader-png.so` and `im-thai.so`). `RPATH` outranks `LD_LIBRARY_PATH` and cannot be overridden from the environment at all, so the only defence is never letting GTK be pointed at those `.so` files. It also explains the timing — the failure lands when GTK dlopens a module, not at `exec`, so the process starts fine and dies on window creation | Extending the unset list again — the fix that has now failed twice, and it rots on every snap revision. Manipulating `LD_LIBRARY_PATH` — powerless against `RPATH`, and core20 in `LD_LIBRARY_PATH` would break `bash` itself, which is evidence it was never the live mechanism. Requiring an external terminal — rejected in D-16 for the reason it is still rejected: it is a trap that gets rediscovered every few months |
| 2026-08-04 | **D-31** R-13's reveal log lives in a new `audit` key **inside the sealed body**, buffered in core memory and flushed on save, capped at **1000 entries** pruned oldest-first, with the setting **off by default** | Three sub-decisions, one entry, because they only make sense together. **Location:** the format spec's §9 already says a new body key is not a format change, so v1 absorbs it with no `format_version` bump and no KAT regeneration — and a record of *which* secret was revealed *when* is sensitive enough that it belongs behind the same seal as the secrets. **Flush on save, not on reveal:** flushing per reveal means a full re-encrypt, fsync, and rename every time a password is read, which changes the file's mtime and inode on an operation the user thinks of as a read. Cost carried: a crash loses the un-flushed tail. Acceptable because the log's reader is the vault's owner reviewing their own habits, not a forensic investigator — an attacker holding the master key can rewrite it anyway, so it was never tamper-evidence. **Count cap, no age cap:** bounds file growth, which is the failure that actually degrades the product. Risk carried and named rather than hidden: for a light user 1000 entries may reach back years, so a cracked vault reveals a longer history of habits than an age cap would allow. **Off by default:** the author's call. It departs from `MASTER.md` § *Reveal*, which states "Reveal on a masked field emits an audit-log entry" unconditionally — that line now describes the behaviour when the setting is on, and the Settings copy must not imply the log is running when it is not. **No prior art exists to copy:** 1Password's audit log is server-side and Business-only; KeePassXC has no reveal log at all and [issue #5573](https://github.com/keepassxreboot/keepassxc/issues/5573) requesting one is still open. Finding nothing is the finding — this shape is being invented here, so it gets the conservative default | Plaintext sidecar file — a readable list of which secrets were revealed and when, which is precisely what the vault exists to prevent; never seriously on the table. Encrypted sidecar `<vault>.audit` — leaves the vault file untouched by reads, rejected because it is a second format and a second seal to version, and copying the `.tvault` alone (the entire point of a single portable file) would silently drop the log. Flush on every reveal — durable to the entry, rejected for the write amplification above. Age cap, or age **and** count — the stronger privacy answer, rejected by the author in favour of the simpler rule; the gap is written down above rather than lost. On by default — matches `MASTER.md`'s wording and means the log is non-empty on the day it is first wanted, rejected for the stricter privacy default |
| 2026-08-04 | **D-32** The mask on an elided secret field is a **fixed 12 characters**, never the secret's true length | Written while specifying `list_items`, and it is the kind of thing that gets "fixed" later by someone making the dots look right. `"•".repeat(value.len())` puts the exact length of every password in the vault into a heap that cannot be wiped, for every item in the list, with no user action and nothing revealed. Length alone is not catastrophic, but it is the one component of a secret that leaks for free, it narrows a search space, and it buys nothing but dots of a pleasing width. The fixed width is a deliberate lie and `docs/ipc-contract.md` §6.2 says so, so the UI cannot present it as a length | True length — what the design's mockups imply and what any implementer reaches for first. A random width per field — hides the length but makes the row jump between renders, and a mask that changes when nothing changed reads as a bug. No mask at all, just the label — loses the affordance that tells the user there is something there to reveal |
| 2026-08-04 | **D-33** Settings live in **plain JSON in the OS app-config directory**, not in the sealed body — all four of them, including `audit_log_enabled` | Answers the open question that writing `docs/ipc-contract.md` §6.3 raised. Two constraints decide it. `theme` must be readable **before** any vault is open, or the lock screen paints in the wrong colours for as long as the unlock takes — that alone rules the sealed body out for at least one value. And none of the four is a secret: a theme, two durations, and a boolean. The one that looks like it belongs in the vault, `audit_log_enabled`, does not: the *log* is sensitive and stays sealed, but the **switch** governing it is not, and it has to be readable at the moment a reveal happens without a second lookup path. Splitting the four across two stores to keep one boolean company would mean two formats, two migration stories, and a rule nobody remembers | All four in the sealed body — rejected because the theme is then unreadable at the moment it is needed. Split: theme and auto-lock in a config file, audit and clipboard in the vault — the tidiest-looking answer and the worst one, two stores for four values. A platform keychain — a per-OS trusted path for data that is not secret, which is cost with no benefit. Not persisting at all and keeping them in memory — considered seriously, since it defers the question, and rejected because an app that forgets your theme every launch is one the author would fix within a day and then not write down |
| 2026-08-04 | **D-34** Where the prototype and `MASTER.md` disagree on a *number*, three deviations are taken and written down rather than resolved silently | The rule in `CLAUDE.md` is that `MASTER.md` wins and the disagreement is logged, so this row is the log. **(a) Icon buttons and chips are 28px, not the prototype's 26px** — §4 sets the pointer-fine target floor at 28px and §7 puts a button at 28px tall, so the prototype's own toolbar is 2px under its own rule; the token is `--control-h` and the whole app reads it, so changing the mind later is one line. **(b) An empty strength segment is `--border-strong`, not §2's `--fg-subtle`** — this one goes the *prototype's* way, because at 4px tall `--fg-subtle` reads as a filled segment and a meter you cannot count is not a meter; §2's intent ("four discrete segments") is better served by the darker value than by its literal text. **(c) The filled segments take one colour chosen by count** (1–2 danger, 3 warn, 4 ok) rather than a per-index ramp, so a two-segment bar is not half-reassuring | Following the prototype to 26px everywhere — smaller and denser, and it puts every icon button under the app's own minimum. Following §2 literally on the empty segment — measured on screen and rejected. Resolving any of the three in code without a row here, which is how "already checked" becomes indistinguishable from "never considered" |
| 2026-08-04 | **D-35** The recovery flow's **step 2 shows the freshly issued kit**; it does not set a new master password as the prototype does | `unlock_recovery_kit` unlocks with the kit and reissues one, because using a kit spends it. The prototype's step 2 sets a new master password, which needs a change-password command that does not exist and is not in Phase 2's scope. The two-step shape, the 420px dialog, the header's "Step *n* of 2" and the footer are the prototype's unchanged; only what step 2 *carries* differs, and what it carries is the thing that actually happened. The alternative would have been a step that collects a password and throws it away | Drawing the prototype's password form and disabling it — a form that cannot be submitted on the one screen a locked-out user reaches for. Collapsing recovery to one step — loses the reissued kit, which is the whole reason the flow cannot end at "unlocked". Building a change-password command to match the drawing — scope creep across a gate, decided by a mockup rather than by a requirement |
| 2026-08-04 | **D-36** Surfaces the design draws but Phase 2 has no command for are **built and visibly disabled**, not omitted | Directed by the author when the UI-parity work was scoped: the whole prototype, with dummy data where a command is missing. The alternative shapes the app's silhouette — a titlebar with no ⌘K, a list header with no `+`, a detail pane with no toolbar all read as *this product does not have that*, which is a different and wrong promise from *not yet*. Every disabled control carries the reason in its `title`, so the gap is legible from the UI and not only from this table. Cost carried and named: the app now contains chrome that does nothing, and the Phase 3 risk is that a disabled button gets wired to a stub instead of a command. Specifically drawn-but-inert: Edit, Delete, New item, Rename vault, Open vault file…, New vault, Load recovery kit PDF…, and both of the generator's copy paths | Omitting them until the command lands — the honest-looking option, rejected by the author and, on reflection, the less honest one about what the product is. Wiring them to frontend stubs — the shape that makes a Phase 3 reviewer believe the feature exists. Shipping a separate design-only preview build — two UIs to keep in step, which is how they diverge |
| 2026-08-04 | **D-37** The password generator's preview is minted in the webview with `crypto.getRandomValues`, and **cannot be copied from that dialog** | The generator is drawn in full because D-36 says so, but the copy paths stay closed, and the reason is the rule the whole architecture rests on: the clipboard clear that makes a copy safe is scheduled by Rust inside `copy_field`, so a copy from this dialog would leave a password in the clipboard that nothing ever clears — worse than the app's own promise. The value itself is generated properly rather than faked: `crypto.getRandomValues` with rejection sampling, never `Math.random` (which the prototype uses) and never modulo bias, so nobody can later promote a toy into the real one by not noticing. `generate_password` remains the fourth sanctioned command `docs/ipc-contract.md` already names, and adding it is still a decision rather than a patch | A fixed placeholder string — visibly not a password, and it makes the length slider and the character-set chips inert theatre. `Math.random`, as the prototype does — a seeded PRNG with recoverable state, one careless promotion away from being the shipped generator. Enabling copy through `navigator.clipboard` — the secret then sits in the clipboard with no auto-clear, which is precisely the failure the chip on the detail pane promises does not happen |
| 2026-08-05 | **D-38** The Phase 2 gate's functional line is **re-worded**; "read an item" moves to the Phase 3 gate | The line was written at kickoff against a phase boundary drawn later. `add_item` is Phase 3, Phase 2 deliberately shipped no mutation command, so a vault created through onboarding is empty and there is nothing to read. Directed by the author. Risk carried and written into the phase document rather than absorbed: G-B′ can now pass without a human having watched a secret cross IPC in the running app. What covers that path instead is `tests/ipc_audit.rs` plus the new `tests/ipc_session.rs`, both automated and both run on every push; what stays unproven until Phase 3 is the *frontend* half — that the detail pane renders a real item and `SecretField` reveals it | Pulling a minimal `add_item` into Phase 2 — meets the line literally, and is scope creep across a gate: it drags in the New-item dialog, validation, and seven item types, all Phase 3 by the roadmap. Seeding a vault through the core's API and reading *that* through the UI — proves the boundary but not "through onboarding", so it would have been a gate line met by a fixture; it survives as the mechanism inside `ipc_session.rs`, where it is honest about what it is |
| 2026-08-05 | **D-39** R-10's acceptance criterion is corrected to name `src-tauri/tests/ipc_audit.rs` | It read "enforced by a **core** test", which is impossible: N-02 forbids `trustvault-core` from depending on Tauri, so the core cannot see a command at all. The correction names the file that can and does enforce it — `reveal_returns_one_secret_and_copy_returns_none` asserts exactly one occurrence per invocation — and says *why* in the requirement text, so a later reader cannot mistake it for a weakening | Leaving the criterion as written — it stays unmeetable and the gate inherits a line nobody can satisfy. Deleting the criterion — R-10 then has no acceptance test at all, which is worse than one in the wrong crate |
| 2026-08-05 | **D-40** The last-opened vault path is persisted in the settings file, and restored at start-up | Found by `tests/ipc_session.rs` on its first run, which is the second time a harness has caught drift on the day it was written. The host keeps nothing across a quit, so `vault_status` answered `no_vault` after a relaunch and the frontend routed to **onboarding** — a user who created a vault yesterday was shown the create-a-vault flow today, with no way back to their file, because "Open vault file…" is drawn-but-inert (D-36). It is not a secret: a path to a file whose entire security is that it is encrypted, stored beside the theme. Only the path is restored, never a key, so the state it produces is `locked`, which is exactly what a relaunch should show. A remembered path that no longer resolves yields onboarding and the setting is **left alone** — forgetting the vault because a USB stick was unplugged once is the wrong direction to fail | A separate window-state file — a second format and a second migration story for one string, against D-33's own argument. Storing it in the sealed body — unreadable at exactly the moment it is needed, the same trap the theme has. Scanning the documents directory for `*.tvault` — guesses, and it means the app opens a file the user did not choose. Letting the webview set the field through `set_settings` — rejected in code as well as here: `merge_incoming` overwrites whatever arrives, because `Settings` deserializes with defaults and a frontend that does not know the field would erase it on every theme change |
| 2026-08-05 | **D-41** The clipboard chip promises **TrustVault's own copy** and nothing more | The open question is answered by experiment, and the answer is no. Measured against GPaste 45.3 on GNOME/Wayland with `track-changes` on: it recorded the copied value **with** `x-kde-passwordManagerHint` set, and still held it after `clipboard::clear()` ran. The hint is the most widely adopted Linux convention and it was D-11's whole reason for choosing `arboard` — going to look is also how we found the hint **was not actually being sent**; `clipboard::set` now sends it, and GPaste ignores it anyway. So the chip reads "TrustVault clears its copy in *n*s", with the rest in its `title`. `tests/clipboard_manager.rs` pins the finding and fails if clearing ever *does* reach a manager's history, which would be the day the wording may be strengthened | Keeping "Clipboard clears in 12s" — measurably false on the tester's own desktop, and the one claim a password manager must not get wrong. Dropping the chip — the auto-clear is real and worth showing; silence would understate it as badly as the old wording overstated it. Refusing to copy at all while a manager is running — unenforceable (nothing can enumerate them) and it breaks the product's main verb |
| 2026-08-05 | **D-42** v1 imports from **Bitwarden's unencrypted JSON export and nothing else**; CSV, KDBX, `.1pux`, every vendor's encrypted export, and export *from* TrustVault are out of scope, each with its own line in `trustvault-project.md` | Closes R-29 ahead of Phase 3's entry check, which names it as an external dependency that must be resolved *before* the first task because it changes the item model. Bitwarden's JSON is the only surveyed candidate that carries **TOTP seeds, folders, custom fields and item types** in a documented, stable schema while costing one pure-Rust dependency (`serde_json`) — no XML, no zip, no second crypto stack. That matters because N-02 puts every plaintext byte in `trustvault-core`, so the importer parses **externally supplied, potentially hostile input** inside a Tier-1 crate under `forbid(unsafe_code)`, a no-panic lint set and a 90 % coverage floor; the smaller the parser, the more of that is real. The survey's other finding shapes the requirement rather than the format: the documented failure of *every* importer looked at is a **field dropped in silence** — folders ignored, TOTP seeds landing in a note or nowhere, attachments vanishing. So R-29's acceptance criterion is not "it imports" but "every field is either mapped or **named in a refusal**", and the import is one transaction with a preview | **CSV** (from any manager or browser) — the widest reach and the only format some vendors emit, rejected because it has no schema: an importer either guesses headers or needs a column-mapping surface Phase 3 does not carry, and TOTP seeds are usually absent from CSV exports entirely. **KDBX via `keepass-rs`** (0.13.6, May 2026, actively maintained — it would work) — rejected for what it drags into a Tier-1 crate whose entire dependency tree is today serde, thiserror and zeroize: AES, Twofish, ChaCha20, Argon2 and a full XML parser. **`.1pux`** — a zip plus a JSON blob whose schema is documented by third parties rather than by 1Password, so it can change without notice. **Any vendor's encrypted export** — means implementing their KDF and key-stretching pipeline, crypto surface with no upside when the same app emits a plaintext export in one click. **Out of scope entirely** — honest and cheap, rejected because a vault nobody can migrate into is a vault nobody adopts, which was R-29's original argument |
| 2026-08-05 | **D-43** Custom fields get **one stored bit on `Field`** (`custom`, absent when false); folders get **nothing at all** and become tags | D-42's unsettled consequence, and the first task of Phase 3 because the importer decides it by accident otherwise. **Custom fields:** the flag is stored for the reason `secret` is stored (§6.3) one step along — inferring it from "is this label in the type's standard set?" fails in both directions on real data, and the dangerous direction is that an imported custom field called "Username" becomes the login's own and overwrites a real credential into `history`, where no v1 surface can reach it. So `set_field` searches only non-custom fields and the importer uses `push_field`, which never merges: Bitwarden permits two custom fields with the same name, and collapsing them is a field dropped in silence — the exact failure R-29 refuses to call an import. Written as `skip_serializing_if`, so a vault with no custom field encodes to the bytes it did before the key existed and the KAT vectors stay valid unregenerated. **Folders:** a folder is single-parent, `tags` is many-to-many, every folder is expressible as a tag and not the reverse — a `folder` key would be a second taxonomy over the same items, and the sidebar would owe two filters meaning nearly the same thing | **Custom fields:** a `custom_fields` map beside `fields` — a second container with its own ordering, elision, and history rules, all of which `Field` already has. Deriving customness from a per-kind label schema — the inference §6.3 already rejects, in a new place. Nothing at all, letting custom fields land as ordinary fields — indistinguishable in the detail pane and colliding with the type's own on label. **Folders:** a `folder: Option<String>` key — two taxonomies, and the one that cannot express what the other can. Flattening a nested path to its leaf (`Clients`) — collides across parents; splitting it into two tags — claims a hierarchy tags do not have. `Work/Clients` is kept verbatim as one tag |
| 2026-08-05 | **D-44** `generate_password` is the **fourth sanctioned command**, returning one `Secret`; copying from the generator goes through a new ambient `copy_generated` | The contract's §10 named this as needing a decision and named the alternative to consider first, so this is that decision rather than a patch. Three arguments in order. **It must return the value** because the surface the design draws shows the password with a regenerate button beside it — the user is deciding whether to accept *this* one; a generator whose output can only be pasted makes the length slider and the set chips into theatre and cannot fill the New-item dialog's password field at all. **That is not a widening**: a password being generated is not yet a stored secret, it protects nothing until saved, and an item created by typing a password by hand puts the identical string in the identical unwipeable heap — §5 already concedes inbound is a direction this contract does not defend. **And it buys the thing that matters**: randomness for stored credentials moves onto the one path R-06 and D-23 already constrain (`getrandom`, one file, a CI grep forbidding any seedable RNG), instead of `crypto.getRandomValues` in the webview. D-37's constraint is honoured rather than repealed — what D-37 objected to was a copy nothing would ever clear, and `copy_generated` schedules the same clear `copy_field` does. Costs carried: the sanctioned count is now 4, `ipc_audit.rs` asserts the number *and* the existence of this row, and `copy_generated` is an ambient command that takes a secret inbound — safe only because it reads nothing and returns nothing, which is written into the contract beside it | Generating **straight into the clipboard**, never crossing IPC — §10's own suggestion and the strongest alternative: nothing to leak, and `copy_field` already proves the pattern. Rejected because the value is then never visible, which is not the surface being built. **Keeping the webview generator** (D-37, `crypto.getRandomValues` with rejection sampling) now that it can be saved — two generators, one of them "the real one", a distinction that lasts exactly as long as the person who remembers it. Returning the password **without** the score, and re-scoring through `score_password` — sends the value across the boundary a second time for a number the generating side already has |
| 2026-08-05 | **D-45** A TOTP code is **not** a `Secret`; the line is drawn at the seed | §10 left this open with the right warning attached — *"it expires soon" is the argument that ends with secrets in lists* — so the decision deliberately does **not** rest on the lifetime, which would equally license returning a password about to be rotated. It rests on three properties a password does not have: a code cannot be run backwards to the seed, so a code stranded in the webview heap discloses one 30-second window that has already passed; the protocol's own operation is to transmit it to a remote party, so it is a secret whose intended use is publication; and it is single-use and self-invalidating. Bounded explicitly so it cannot be read as broader: `totp_code` serves **one item at a time, the selected one**, never batched and never in a list, and the seed stays a `secret: true` field revealable only through `reveal_field`. `ipc_audit.rs` check 7 pins both halves | Marking it `Secret` and making it the **fifth** sanctioned command — defensible, and it makes the countdown ring in the detail pane a sanctioned call on a 30-second timer, which breaks §2's "only on explicit user action" rule outright; the rule would have had to be weakened to accommodate it. Returning the **seed** to the webview and generating codes in JS — one crossing instead of one per 30 s, and it hands over the credential itself to save arithmetic. Codes in `list_items` so the list can show them — the exact shape §2 exists to prevent |
| 2026-08-05 | **D-46** The command palette's fuzzy search runs in **Rust** (`search_items`), not over a client-side copy | R-16 asks the palette to search titles, usernames, URLs and tags, and only two of those are in `ItemSummary`. Getting the other two into JavaScript is a *permitted* crossing — §6.1 says a non-secret field's value crosses freely, and usernames and URLs are non-secret by the user's own declaration — which is what makes this worth a row: the rule allows it and it is still the wrong trade. It would put the entire identifying surface of the vault into a heap that cannot be wiped, on every list render, for a feature used a few times a day, to save one IPC round trip against a 50 ms p95 budget (S-04) with room for it. So the query goes inbound, the matching happens against plaintext that never leaves the core, and what returns is the same elided summary the list already gets — **including no record of what matched** | Widening `ItemSummary` with `username` and `url` — the obvious implementation, rejected above. Adding a `subtitle` or `search_terms` projection — the same leak with a name that hides it. Returning the matched value so the palette can show context under each row — deferred rather than refused: if the design needs it, it covers the visible results only and gets its own decision, instead of arriving as a widened return type |
| 2026-08-06 | **D-47** Every Tauri command carries `#[tauri::command(rename_all = "snake_case")]`; the wire format stays `snake_case` as `docs/ipc-contract.md` prints it | Not a preference — a **live bug**, found while wiring the mutation commands and confirmed by reading the built binary (`strings target/debug/trustvault` showed `reveal_field…itemIdfieldId`). Tauri v2's `#[tauri::command]` renames arguments to **camelCase** by default and `tauri::ipc::CommandItem` looks the key up **exactly, with no fallback**, so the host was asking for `itemId` while `src/lib/ipc.ts` sent `item_id`. Every command with a two-word argument was unreachable from the webview: `get_item`, `reveal_field`, `copy_field` — three of the four things the detail pane does. It survived Phases 2 and 3-to-date because every command exercised until now happens to have single-word arguments (`path`, `password`, `name`, `settings`), and because the `_inner` split that lets `ipc_audit.rs` drive the **real** command bodies skips argument decoding — the harness is well-shaped for what it was built for and structurally blind to this. Asserted now on **every** command rather than the ones that need it today, because the expensive version of this bug is the one reintroduced by adding a second word to an argument name. `src/lib/ipc.ts`'s `snake()` was also made to descend into arrays in the same commit: `fields: EditField[]` is the first array of objects to cross outbound, and the omission would have become a silently dropped key on the day one of them was named with two words | **Renaming in the frontend instead** — drop `snake()` for arguments, send camelCase, and rewrite §6 of the contract to match. Rejected because the contract is the artefact this project defends: `ipc.ts` says in its own comment that the conversion is there *so the wire format stays exactly what the contract documents*, and changing the document to match a default nobody chose inverts that. **`rename_all` only where a two-word argument exists today** — smaller diff, and it makes the next two-word argument a silent regression. **Leaving it and having the frontend send camelCase ad hoc** — two naming conventions on one boundary, which is how the next person gets it wrong |
| 2026-08-06 | **D-48** The Edit-item dialog is **field-driven**, built from what `get_item` returned, not from the type's template | The prototype **draws no edit surface at all** — its detail-pane Edit button has no handler — so this is the first surface in the project with no pixel values to follow, and `MASTER.md` is the only authority, which is the case `CLAUDE.md` names. The shape is forced by the contract rather than chosen: `update_item`'s third rule is that **omission deletes**, and an item is a *list of fields*, not an instance of a template. A form built from `ITEM_FIELDS[kind]` would submit a list missing every field the template does not know about — every custom field an import created (D-43) — and delete them, in silence, on a rename. The second rule gets a surface of its own for the same reason: a secret row shows its **mask** (fixed twelve characters, D-32, never the length) with a **Replace** button, and sends `value: null` until Replace is pressed, so *unchanged* is something the user can see rather than something the code remembers | **Reusing the New-item dialog in an edit mode** — one surface instead of two, and the obvious first move; rejected for the silent deletion above, which is precisely the failure R-29 refuses to call an import. **A `remove_field` command so the form could send only what changed** — makes an edit two round trips that can half-succeed, and §6.4 already rejected it. **Prefilling secret rows with the mask** — what a naïve implementation does, and it writes `"••••••••••••"` into every password in the item on the first rename; it is the exact failure `value: null` exists to prevent, so the form holds `null` end to end rather than translating at the last moment |
| 2026-08-06 | **D-49** The delete confirmation says the item is removed **straight away, with no Trash and no undo** | The copy said *"It goes to Trash for 30 days first"*, written against a Trash view that filters to nothing and a `delete_item` that removes the item, saves, and zeroizes the dropped value. It was false on the day it was written and it is the worst possible sentence to have wrong: it is the last thing a user reads before clicking, and it is the reason they click. The Trash **view** stays drawn under D-36's rule — an absent Trash reads as "this product does not have one" — but the promise attached to the button is now what the command does | **Building a Trash so the copy becomes true** — a soft-delete flag, a retention timer, and a restore path, none of them in Phase 3's scope and all of them decided by a sentence rather than by a requirement. **Removing the Trash view as well** — it is drawn under D-36 and the rule there has not changed. **Leaving the copy and fixing it "when Trash lands"** — the version where the app lies for however long that takes |
| 2026-08-06 | **D-50** A Bitwarden item of a type TrustVault does not have is imported as a **secure note carrying its own keys as custom fields**, through one generic path, rather than being refused | Reading `CipherType` in `bitwarden/clients` rather than trusting the number everyone quotes found **eight** types, not four: 5 is an SSH key and 6, 7 and 8 are a bank account, a driving licence and a passport. Three of them have no home here and there will be a ninth. Refusing them is honest and loses the data outright, at the exact moment a user is migrating *away* from the other product — the one moment they cannot go back and copy it by hand. So the object is walked generically: string, number and boolean values become custom fields under their own keys, a list or a nested object is refused by name, and the item carries a `converted` entry saying what it was. Written generically on purpose — three hand-written mappings would each need the schema read again, and the fourth type would arrive as a silent hole. Cost carried and named in the code: nothing is known about what an unknown type holds, so its string fields are stored **masked**, because leaving an account number in the clear is the failure this product exists to prevent and masking a house number is an annoyance | **Refusing the item** — the shape the task list assumed, and the one that loses data. **A new `ItemKind` per Bitwarden type** — a format change decided by another vendor's release notes, and D-13's argument against absorbing decisions into patches applies twice. **Mapping them onto our nearest type** (a bank account as a card) — guessing, which is what D-43 built the `custom` bit to stop, and the guess would be wrong for a driving licence. **Importing only the four types everyone quotes** — what a parser written from memory would have done, and it would have dropped SSH keys, which we *do* have a type for |
| 2026-08-06 | **D-51** An **unknown key** in an export is a refusal, not something skipped | R-29's criterion is that every field is either mapped or named, and the failure it guards against is a *future* schema, not today's. Bitwarden ships a client release every few weeks; `archivedDate` and `key` are both newer than the four-type schema most write-ups describe. So every key is declared in the parse structures — including five that are declared and deliberately **not** imported, each with its reason in the module documentation — and anything left over lands in a `#[serde(flatten)]` map that becomes one refusal per key. `null`, an empty list, an empty object and an empty string are skipped, because a report full of noise is a report nobody reads, which fails the requirement by a slower route. Cost carried: an export from a much newer Bitwarden will produce refusals for keys that do not matter, and the fix is to declare them here rather than to loosen the rule | **Ignoring unknown keys**, which is what `serde` does by default and what every importer surveyed for D-42 effectively does — it is the mechanism behind "a field dropped in silence", and choosing it would have made the requirement untestable. **Failing the import on an unknown key** — safe, and it means a Bitwarden update breaks importing entirely until we ship. **Listing keys we know to ignore and refusing the rest** — the same thing, written as a list that goes stale; the declaration *is* the list, and it lives beside the field it describes |
| 2026-08-06 | **D-52** The generator surface departs from the prototype twice: the footer reads **Copy password**, not *Copy & autofill*, and ambiguous-glyph exclusion is **permanent rather than a toggle** | Both are `MASTER.md`-over-prototype calls of the kind `CLAUDE.md` names, and both are about a control promising something. **Autofill:** it is a browser extension with native messaging behind it, and `trustvault-project.md` puts it out of scope for v1 — a primary button offering it is the same class of mistake as D-49's "It goes to Trash for 30 days first", found one screen away and two days earlier. **Ambiguity:** the prototype's own digit chip is labelled *2–9*, not *0–9*, so the design already treats the exclusion as a property of the character sets; a fifth chip switching it off would make all four existing labels lies whenever it was off, on the one surface `MASTER.md` §3 says must survive hand-transcription. The parameter still exists in the core and in the contract because that is where the alphabet lives and where a future setting would read from — what is fixed is the **UI**, not the capability. Exactly the five glyphs §3 names (`0 O 1 l I`): the retired webview preview also dropped lowercase `o`, which buys nothing once `0` and `O` are both gone and costs entropy for free | **Renaming the button *Copy & fill*** and having it write into the New-item dialog — plausible, and it is a second meaning for a dialog that can be opened from anywhere; the fill path already exists as the *Generate* button inside that dialog. **Leaving *Copy & autofill* and disabling it** — D-36's rule covers surfaces the design draws that have no command yet, and this one has no *feature* behind it in v1 at all, which is a different thing: it would read as "coming soon" for something deliberately out of scope. **A fifth chip for ambiguity** — the honest-looking option and the one that breaks the other four labels. **Keeping `exclude_ambiguous` out of the contract entirely** now that nothing toggles it — rejected because the core is where the alphabet is decided, and a hard-coded filter there is the version nobody can find later |
| 2026-08-06 | **D-53** The palette searches **every field the user did not mark secret**, and `secret` is the only gate — not the field's `kind` | Written first as R-16's own list (title, tags, and non-secret fields of kind `username`/`url`/`email`) and falsified inside the hour by the IPC harness's fixture: `Item::set_field` sets `kind` from `secret`, so a username stored through it is a `text` and was not searchable. The two attributes are not interchangeable — **`kind` is how a field renders and is only as accurate as whoever created it, `secret` is what the user declared** — and only one of them is a security boundary. Filtering on `kind` produces a failure nobody can diagnose from the outside ("why does this item not come up when I type its account name?") and no test would catch, because every test would have set the kind correctly. So the rule is one line: a secret value is never a haystack, everything else is. That is deliberately **broader than R-16 asks**, and the cost is named rather than hidden — a long note body is now searchable, so what stops it outranking a title is the scoring weights (title 300, tag 200, field 100) rather than the haystack list, and there is a test named for exactly that. The security half is the reason the rule is absolute and not a filter anyone may extend: a palette that matched stored passwords would confirm a guessed one **through the ranking alone** — nothing revealed, nothing crossing IPC, no audit entry, answer on screen. Pinned in both crates: the core asserts it for every `FieldKind`, `ipc_audit.rs` asserts a query equal to the planted secret returns nothing | **Kind-filtered, matching R-16 word for word** — what the contract said first; it makes searchability depend on metadata the application itself gets wrong, and the demonstration was our own fixture. **Fixing `set_field` to infer `kind` from the label instead** — the inference D-43 built the `custom` bit to stop, in a new place, and it would still leave every custom field's kind a guess. **Adding a `searchable` flag to `Field`** — a third attribute meaning almost what `secret` already means, stored in the vault, and wrong the first time somebody sets it by hand. **Searching secret values too, ranked below everything else** — never on the table; it is the oracle above, with a ranking penalty as its only defence |
| 2026-08-06 | **D-54** TOTP is built on `hmac` 0.13 + `sha1` 0.11 + `sha2` 0.11 (RustCrypto), with RFC 6238 written in `trustvault-core/src/totp.rs` — not on a TOTP crate | The forty lines above HMAC are **pinned by numbers upstream published**: RFC 6238 Appendix B gives 18 vectors across three algorithms, so there is no judgement in this arithmetic for a library to have exercised better. What a library would add is what it drags with it. The `zeroize` feature is on for all three and is the reason the **0.13/0.11 line** was chosen over the older 0.12/0.10 one, which would have reused the `digest 0.10` argon2 already pulls through blake2: the HMAC state holds the seed, N-01 says key material is wiped on drop, and the 0.10 line has no zeroize wiring at all. The cost is named rather than hidden — **two `digest` versions in a Tier-1 crate's tree** until argon2 0.6 moves us to 0.11 anyway (the Cargo.toml comment on argon2 already tracks that release). MSRV 1.85 on all three, under our 1.88. Both licences are MIT/Apache-2.0, and `sha1 0.10.7` shipping *after* 0.11.0 shows the family is maintained across lines rather than abandoned behind them | **`totp-rs` 5.7.2** — the obvious choice, actively released (5.7.2 on 2026-06-23), and it wraps exactly these three crates. Rejected for what comes with it rather than for what it is: a second base32 implementation (`base32`) beside the `data-encoding` the recovery kit already uses, a second constant-time compare (`constant_time_eq`) beside `subtle`, and `otpauth://` parsing behind a feature that pulls `url` **and** `urlencoding` — two crates to split a query string whose four keys we control. Its `TOTP.secret` is a public `Vec<u8>` whose own docstring says "sensitive data, treat it accordingly", zeroized only with the feature on; the core's idiom is that a seed is not reachable at all. **`totp-rfc` 0.1.2** — a 0.x with three releases and a single maintainer; nothing wrong with it, but a Tier-1 crate does not take a key algorithm from a dependency with no track record, which is the same rule that keeps argon2 at 0.5 rather than its 0.6 RC. **Hand-writing HMAC-SHA-1** — never on the table: the primitive comes from an audited implementation, which is what D-21…D-24 decided once. **Supporting SHA-1 only** and refusing the other two — smaller, and silently wrong for any issuer using SHA-256, which `otpauth://` URIs in the wild do carry |
| 2026-08-06 | **D-55** The Add dialog does **not** draw the prototype's *Scan QR on screen* button | Reading a QR code off the display needs **screen capture**, a permission a password manager is better off never holding and one v1 does not ask for anywhere else. It is the same class of false promise as D-52's *Copy & autofill* and D-49's "It goes to Trash for 30 days first" — a control that describes a capability the build does not have. The field beside it already accepts both shapes a user can actually get at (a base32 key, or the whole `otpauth://` link a QR encodes), so what is lost is the convenience of not switching windows, not the capability. This is the **third** prototype promise removed in three days, which is worth reading as a pattern rather than three incidents: the design was drawn before the scope table existed | **Drawing it disabled** — D-36's rule is for surfaces whose command has not landed yet; this one has no feature behind it in v1 at all, and a disabled control reads as "coming soon" for something deliberately out of scope. **Shipping it with a file picker instead** ("choose a QR image") — a different feature wearing the same label, and it needs the file-picker dependency the import surface is still waiting on. **Keeping it and opening a "not in v1" note** — a button whose entire behaviour is an apology |
| 2026-08-06 | **D-56** `launch_at_login` is **hand-written per platform** in `src-tauri/src/autostart.rs` — an XDG desktop entry, a `LaunchAgent` plist, a `reg.exe` value — rather than taken from `tauri-plugin-autostart` or the `auto-launch` crate under it | The manifest's standing rule is that a plugin arrives when a requirement needs one and not before, because a plugin is widened attack surface in a process holding decrypted secrets — and here a requirement (R-21) genuinely does need one, so the rule does not settle it and this row is the decision the rule asks for. What settles it is the **size of the thing being bought**: three file writes and one `reg.exe` invocation, about eighty lines, against a dependency whose own transitive tree exists to abstract exactly those three. And the contract already specifies a failure mode neither candidate offers — *report `io` and leave the stored value alone* — so a wrapper would have been wrapped again to get it. Two properties are pinned by test rather than assumed, because both are silent when wrong: **enabling twice is enabling once** (the entry is written, never appended to), and **disabling what is not registered succeeds** (the desired state already holds; an error there makes a fresh install show a failure for a toggle nobody touched). The stored value is also **reconciled against the OS at start-up** — a user who removed the entry through their desktop's own startup tool has said something the settings screen must not go on contradicting. Costs carried and named: only the Linux path is exercised by CI, so macOS and Windows are written-and-unverified until G-C's per-platform re-measurement; and the Windows path spawns `reg.exe`, which is a subprocess from a process holding secrets, bounded to fixed arguments against a binary that ships with the OS | **`tauri-plugin-autostart`** — official, actively maintained, and covers all three platforms; rejected because using it as a plugin registers invoke handlers this app has no use for, and using only its Rust API means depending on a plugin to not be a plugin. **`auto-launch` (the crate beneath it)** — the same coverage without the handlers, and the closest call here; rejected because its macOS path adds a login item through **AppleScript**, which means `osascript` — a script interpreter as a subprocess, from this process, to write a file we can write directly. **A Windows registry crate** — links the Win32 API into the host to set one string. **The Windows Startup folder instead of the registry** — a shortcut there needs COM, and a `.cmd` instead flashes a console window at every login. **Not shipping the setting** — R-21 names it, and it was already absent from the Settings screen with a note saying so |
| 2026-08-06 | **D-57** UI Scale is a **token multiplier** (`--ui-scale` inside every size `calc()`), not root `rem` scaling as `MASTER.md` §3 and the contract both said | Found on the day the setting was wired, by reading `tokens.css` rather than by trusting either document: the file has **no `rem` in it at all**, and neither does anything else in `src/`. Every size is `calc(<px> * var(--ui-scale))`, with `data-ui-scale` set on the root. The outcome is identical — but **only because nothing in the codebase uses `rem`**, which is a condition rather than a fact, and that is the whole reason this needs a row instead of a wording fix. Under `rem` scaling, a size written in `rem` scales with the setting; under a multiplier it does not, so the first `rem` someone writes is the one the setting silently stops reaching. A control that scales most of a screen is harder to diagnose than one that scales none of it, because it reads as a layout bug rather than a broken setting. So the condition is enforced: a CI grep fails the build on any `rem` in `src/`. Both documents are corrected in place with the divergence stated rather than edited to look like they were always right — the fifth time in five Phase 3 groups a document written first was falsified by the thing it described, and the first time the falsified document was `MASTER.md` | **Changing the implementation to match the documents** — set `font-size` on `:root` and express every token in `rem`; it is the mechanism `MASTER.md` names and it removes the condition entirely. Rejected as a whole-token-file rewrite, touching every measurement in the application, to reach a state indistinguishable from today's — with the KAT-equivalent risk that one converted number is wrong and nobody notices until a screen is looked at. Revisit if a `rem` is ever genuinely wanted. **Silently editing `MASTER.md` §3 to say "multiplier"** — the document would then read as though it had always been right, which is the practice this project keeps catching itself avoiding. **Leaving both documents wrong** — they are what the next person builds against |
| 2026-08-06 | **D-58** The `api_key`/`ssh_key` glyph collision is closed with **one** new glyph, `code` (`< / >`) for `api_key` — `ssh_key` keeps `terminal` | `docs/icon-gaps.md` recorded this as two types sharing a substitute and called it the worst gap in the set, which framed the fix as *draw an ssh key glyph*. Reading `MASTER.md` §8 rather than the gap note reverses it: §8 **assigns `terminal` to the ssh key by name**, so it is that type's glyph and never was a substitution. What §8 actually does is enumerate six of the seven item types and omit `api_key` — and a type with nothing assigned to it is exactly how two of them came to share one for two phases. So the gap is a missing *assignment*, not a missing glyph, and one drawing closes it. `code` names the world the credential lives in, which is the pattern §8 already uses when it gives an ssh key a terminal instead of a key. What is lost is stated rather than glossed: at 16px `< / >` reads "code" and does not say *key* at all, so it is the title beside it that says which credential. Drawn in the shipped set's geometry — 24px viewBox, 1.6px stroke, round caps — per the rule in `docs/icon-gaps.md`, which is also where the reasoning lives so the substitution is not re-argued. §8 is corrected in place for both this and for **still naming Lucide**, which D-28 replaced two phases ago and nobody went back for; the same in-place correction D-57 made one section up | **Drawing a second key-shaped glyph for `ssh_key`** — the obvious reading of the gap note, and it leaves the real gap (a type with no assignment) open while putting **three key silhouettes** in one list, told apart by the shape of the bow, which is not what anyone scanning a list looks at. **Borrowing one glyph from Lucide** — forbidden by §8's *one set, no mixing*, and invisible in review, which is why `docs/icon-gaps.md` exists at all. **Re-wording instead of drawing** — the answer used for the Phase 0 theme toggle; there is no wording that makes two identical glyphs distinguishable. **Leaving it** — it was cosmetic until this phase made all seven types creatable, and it stopped being cosmetic on that day |
| 2026-08-06 | **D-59** `tauri-plugin-dialog` is adopted — the **first plugin in this application** — and the webview is granted **none** of it: it is registered in Rust so `commands::picker` can open a dialog from the host, and `capabilities/default.json` stays `core:default` alone | Two Phase 3 tasks wanted a file chosen and one survey answers both — R-29's import, and the switcher's *Open vault file…*, drawn-and-inert since D-36 for exactly this reason. What decides the shape is not which crate but **which process reads the file**. An `<input type="file">` hands the *webview* the bytes, and for an import those bytes are another password manager's plaintext in the one heap `CLAUDE.md` says can never be wiped — so the picker returns a **path** and the host opens the file, which is the same rule §2 states about vault data from the other direction. Between the two host-side options the plugin wins on the part that is not code: it wraps `rfd` 0.16 with the `run_on_main_thread` + `block_on` dance already written and tested on three platforms, it attaches the dialog to our window as a parent, and Tauri's release train keeps `rfd`'s version in step with the toolkit — a hand-rolled version is a thing we maintain across every Tauri upgrade for the saving of one crate we would pull in anyway. The cost is a real widening and it is bounded by a **test rather than by a sentence**: the plugin registers `open`/`save`/`message` as JS commands, all denied because no capability names them, and §9 check 8 asserts the capability list is exactly `["core:default"]` — the widening is one line of JSON away at all times. Two things the survey found that a README would not: it pulls **`tauri-plugin-fs`** into the tree as a library dependency (it does not register the fs commands, but the next `cargo audit` will see the crate), and its init script **replaces `window.alert` and `window.confirm`** in our page — with a `confirm` that returns a **promise**, so `if (confirm(…))` is now always true. Check 9 forbids both identifiers in `src/` and **failed on its first run**: `DeleteDialog.svelte` had a local `confirm` shadowing the global, safe exactly until somebody moved the call, on the one dialog in the product whose success is irreversible | **`rfd` 0.16 directly**, no plugin — one fewer crate and the same dialogs, since the plugin is a wrapper over it. The closest call. Rejected because what the wrapper contains is the platform-specific part: sync Tauri commands run on the **main thread**, where `blocking_pick_file` deadlocks, and macOS needs an `NSApplication` for a truly async dialog — so we would re-derive `run_on_main_thread` + `block_on` + parent-window attachment and own it across Tauri upgrades. **`<input type="file">` or a drag-and-drop target** — no dependency at all, and it is the one option that is *architecturally* wrong rather than merely more work: the webview reads the file. **Granting the webview `dialog:allow-open` and calling the plugin's JS API** — the documented way to use it, one command instead of two, and it hands anything running in the page the ability to open a picker and read the path back. **A typed path in a text field**, as onboarding already does for a new vault — free, and it is what `default_vault_path`'s note in §5 proposed as the alternative to a picker; rejected because "type the full path to your Bitwarden export" is not a migration flow anybody completes. **Deferring the import surface to Phase 4** — the 7-day daily-drive clock cannot start without the author's own passwords in a vault, so deferring it defers the phase's longest pole |
| 2026-08-07 | **D-60** A **third door** on `tauri-plugin-dialog`: `pick_new_vault_path`, a **save** dialog, and the first picker command that takes an argument | D-59 opened two open-dialogs and wrote that a save dialog "is not one of the two doors" — which was a description of what had been built, not a reason not to build it. The D-36 sweep came back for it: onboarding's *Change* had carried "a file picker would mean adding a plugin" in its `title` since D-36, and that sentence died the day D-59 landed. The plugin is already in the tree, so a third door costs no dependency and no capability. What D-59 made load-bearing survives intact and is restated where it can be read: **the frontend cannot change what a dialog is for** — title and filter fixed in Rust — which was never the same claim as "the commands take no arguments". `suggested` pre-fills the file-name field and is reduced host-side to its own `file_name`, so `../../etc/passwd` reaches the dialog as `passwd`, and the user reads the result before confirming it. The command returns a path and writes nothing; `create_vault` is what writes, and a save dialog naming an existing file means the OS has already asked | **Re-wording the disabled `title` instead** — cheapest, and it is the answer the sweep exists to refuse: a control that will never be enabled is a promise, and the fix for a promise is deletion, not a better excuse. **Deleting the button** — the prototype draws it and R-08 asks for "name & location"; typing an absolute path into a text field is the worst keyboard moment in the application. **Granting the webview `dialog:allow-save`** — the same objection D-59 made to `dialog:allow-open`, unchanged. **A no-argument save dialog** — keeps the argument-less property and opens with no file name, on the one screen where the user has just typed the name they want |
| 2026-08-07 | **D-61** Two drawn-and-disabled controls are **removed rather than wired**: *Load recovery kit PDF…* in the recovery dialog, and *Rename vault* in Settings | The D-36 sweep's job is that no control lies, and there are exactly two honest outcomes per control: wire it, or delete it. These two cannot be wired. **The kit has no file format.** R-07's recovery kit is produced by `window.print()`, so what the user holds is whatever *their* print dialog wrote — a PDF whose layout is their platform's, not ours. Reading a key back out of one means text extraction over a document we do not control the shape of, plus a PDF parser in a process holding decrypted secrets, to save typing 24 characters into the field directly above the button. **Rename has no requirement.** The prototype draws no rename, nothing in `trustvault-requirements.md` asks for one, no command implements it — and the display name a user sees is the file's own stem whenever the vault is closed (`display_name_for`), so renaming inside the app would change a label the file name goes on contradicting. This is the D-49/D-55 pattern for the third and fourth time in three days, which makes it a rule rather than three incidents: **a control that promises what v1 does not ship is deleted, and the reason stays in the file where the control was** | **Leaving both disabled with better wording** — the sweep's own definition forbids a Phase 3 reason and would accept a permanent one, so this was available; rejected because neither reason is permanent-and-true, they are both "not built", which is what a disabled control already says and says worse. **Building a kit file format** — a machine-readable `.tvkit` beside the printed page would make loading possible; that is a new artifact in the recovery path, which is the one path that must work when everything else has failed, and it is not Phase 3 scope. **Implementing `rename_vault`** — a core mutation, a command, a contract entry and a surface, for something no requirement asks for, at the end of a phase |
| 2026-08-07 | **D-62** The switcher's *New vault* is wired: the frontend gets **one** piece of routing state (`creating`), and `create_vault` **refuses a path that already exists** (`path_in_use`) while locking the outgoing vault inside `create_vault_inner` | The last control the sweep found, and the only one whose absence was a hole rather than a blemish: `vault_status` answers `no_vault` exactly once in a vault's life, so onboarding was unreachable ever after and **a user with one vault could never create a second** — R-22's "multiple vaults shall be openable, switchable" reachable only by somebody who already had another `.tvault` from elsewhere. `creating` is the one thing the frontend decides, and it is bounded on purpose: it can show the create flow, it cannot show the shell, and lock state stays the host's (§9 check 6) because asking for a screen is not a claim about whether a vault is open. **The refusal is the important half.** `create_vault` writes the file whole and onboarding opens with the same default name every time, so a user keeping "Personal Vault" for both would have had the first vault written over by the second — no confirmation, no undo, and no key in memory to have warned with. `path_in_use` is the first refusal in this product that protects a file the user is **not looking at**; the default path now steps to `-2` rather than walking into it; and the ordering inside `create_vault_inner` is testable because it is in the `_inner` half — everything that can fail runs first, so a bad path leaves the open vault alone, then the outgoing vault's audit tail is **flushed** and its key zeroized before the new one is installed. The flush is the half that would have gone missing in silence: reveals buffer in memory (D-31), and a vault replaced without one loses the record that they happened | **Removing the button and logging the gap** — the D-61 treatment, defensible and what `trustvault-project.md`'s in-scope list technically supports (it names onboarding for creation and a *switcher* for multiple vaults); rejected because it makes a `must` requirement unreachable by design, which is a worse thing to ship than an unbuilt button. **Overwriting silently** — what the code did before the check, and nobody had noticed because the flow was unreachable. **Asking the user to confirm the overwrite** — a dialog whose yes destroys a vault the app is not showing them; the OS save dialog's own overwrite prompt is the only confirmation that happens where the file is visible. **Checking the path in the frontend** — the webview does not know what is on disk, and a second opinion about the same question is D-59's own rejected shape. **Putting the lock in the `#[tauri::command]` wrapper**, as `switch_vault` does — exactly the layer `delete_vault`'s lesson says no harness can read |
| 2026-08-07 | **D-63** Five colour tokens move so that every text colour clears 4.5:1 on **every background it is drawn on**: `--fg-subtle` in both themes, `--warn` and `--danger` in dark, and the light `--accent` (with its wash) | S-09 and `MASTER.md` §10 have asked for this since kickoff and it had never been measured, because the app needs a Tauri host to render and every claim about how it looks was a claim about its CSS. Measured (`npm run a11y -- --audit contrast`, against the background *actually painted* behind each text node rather than the transparent one on its own element) the first run returned **895 findings**. The token that mattered is `--fg-subtle`: **3.0:1 on a hovered row**, and `MASTER.md` calls it "text disabled" while the application uses it in **69 places** for group labels, counts, metadata and every placeholder — which is content, and WCAG exempts none of it. §2's own contrast figures were all true and **all four were about `--bg-surface`**; a token is read on five backgrounds. What the numbers cost is stated rather than hidden: the light neutral ramp is tighter now, because there is not much room between `#FFFFFF` and 4.5:1 for three distinguishable text levels, and the light brass is a step darker so it clears its own wash — the sidebar's active row is brass text on a brass tint, which is the one pattern in the design that is inherently low-contrast. The measurement is a tool rather than a paragraph, so §2 stops being the record and starts being the summary | **Moving the failing uses to `--fg-muted` instead of moving the token** — arguably the design-correct fix, since §2 does say "disabled"; rejected at 69 call sites, most of which are genuinely third-level content that wants a third level. **Exempting placeholders and empty states** — WCAG exempts disabled *controls*, not placeholder text, and the failures were mostly neither. **Keeping the light accent and recolouring the active nav row's label to `--fg`** — smaller change, and it removes brass from the one place §2 names it for ("active nav"). **Lowering the wash alpha** — computed and it does not work: the accent against a tint of itself barely moves with alpha, which is what made the accent the thing that had to change. **Ticking §10 on the four figures already in §2** — they were true, they were about one background, and the app failed anyway |
| 2026-08-08 | **D-64** The Phase 3 gate's R-29 line is re-worded to *all of **Bitwarden's** item types, producing the five of ours they map onto* — and the fixture is extended to actually cover them | The line asked for "a Bitwarden export covering **all seven item types**" and meant *ours*. No export of Bitwarden's can ever satisfy it: they have no API-key type and no Wi-Fi type, and the only way to reach those two would be guessing from a title, which is the inference D-43 exists to prevent and which `api_key_and_wifi_are_not_reachable_from_a_bitwarden_export` now forbids. A gate's own vocabulary is the author's to change, so it sat as an open question from 2026-08-06 rather than being edited by whoever noticed it. **What the decision cost is the part worth keeping.** The re-wording was proposed in the same session that found the problem, and it said "all *seven* of Bitwarden's" — our number carried across into a sentence about theirs, when `import/bitwarden.rs`'s own comment says **eight**. Acting on it meant asserting the fixture covered them, and it covered **six**: types 7 and 8, the driving licence and the passport, were in no fixture and no test, so D-50's generic path was exercised by the bank account alone. A re-wording taken two days after it was proposed is therefore not a paperwork edit — it was written from the same memory that produced the wrong line, and nothing re-checks it until somebody tries to tick the box. The claim is `the_fixture_covers_every_bitwarden_item_type` rather than a sentence, because "the fixture covers everything" is precisely the assertion that rots the day a type is added upstream | **Keeping the wording and carrying it as a defect into G-C** — honest, and it means Phase 3 never closes cleanly on a line that is met in every respect that can be met; it also leaves the wrong number ("seven" of Bitwarden's) sitting in the open questions to be believed later. **Hand-writing `api_key` and `wifi` entries into the fixture** so it covers all seven of ours — the tempting one, and it produces a file Bitwarden could not emit, which makes the fixture stop being evidence about real exports; reaching those kinds would then need the title heuristic D-43 forbids and an existing test asserts against. **Adding API-key and Wi-Fi to the mapping from some other signal** — same heuristic one level down, invisible once merged. **Dropping the two kinds from our own model** so the numbers agree — the requirement wants seven kinds creatable by hand, and the importer is not the reason they exist |
| 2026-08-08 | **D-65** S-04's end-to-end number is read from a **release** build with an opt-in `measure` feature (`tauri/devtools`), not from `tauri dev`; the feature opens the inspector itself and CI asserts it is never default and never passed by a workflow | The phase document's procedure — *build it, unlock it, type into ⌘K, and read `performance.getEntriesByName`* — does not say which build, and the only build with a console is the one nobody ships. Measured rather than assumed: `cargo bench --bench search --profile dev` reports **6.63 ms p95** for the same matching that costs **0.85 ms** in release, because `[profile.dev.package."*"]` optimizes dependencies and deliberately not our own crates. That is 13 % of the 50 ms budget instead of 1.7 %, spent by a build no user runs, and — this is the part that makes it worth a feature rather than a footnote — a reading taken that way looks exactly like a reading that is not. The same sitting would have produced a number, written it into the gate, and nothing afterwards could tell it from the real one. Two costs are carried deliberately. Devtools is an inspector attached to the process that holds decrypted secrets, so the feature is opt-in and the CI check is what keeps it opt-in: no default features in `src-tauri/Cargo.toml`, and no workflow passing it. And `open_devtools()` is called rather than merely enabled, which makes the line double as the compile-time proof that the feature still reaches Tauri — `open_devtools` exists only under `debug_assertions` or `tauri/devtools`, so a `measure` build that stopped enabling devtools fails to compile instead of launching without a console and wasting the sitting | **Measuring in `tauri dev` and subtracting the difference** — the arithmetic works and the result is a number nobody can reproduce, assembled from two builds. **Measuring in `tauri dev` and accepting it** — passes the gate on the wrong product, and passes it *pessimistically*, which is the version nobody investigates. **`devtools` on unconditionally** — one less flag and a shipped password manager with an inspector, which is R-10's boundary reduced to a right-click. **Optimizing our own crates in the dev profile** so the two builds agree — it would make every `cargo test` slower and unoptimized-debuggable core is what the workspace manifest chose on purpose. **Printing the percentile from the app itself**, behind a Vite flag — no devtools anywhere, and it puts measurement UI in the shipped bundle and a second percentile implementation in the product to be wrong on its own |
| 2026-08-08 | **D-66** An overlay is closed **only if it is still the one on screen** — `Shell.svelte` gets `closeOverlay(which)` and every `onclose` routes through it, rather than the two palette call sites being reordered | The manual keyboard pass found *New item* in the command palette doing nothing, and the defect was not a keyboard defect at all — the pointer path was identically broken, which is why it is a decision here and a finding in `docs/keyboard-audit.md` rather than a failed row. One `overlay` state holds every overlay, so "run the command, then close me" is two synchronous writes to one variable and the close won: `overlay = 'add'` overwritten by `'none'` before a frame. *Generate password* was broken the same way; *Lock vault*, *Watchtower* and *Settings* were not, because they route through `onlock`/`onview` and never touch `overlay` — **three of five command rows worked**, which is exactly why nobody saw it. The empty-state *New item* carried the same two lines in the same order, and it is the worse instance: its own comment explains it exists because a query matching no item has also filtered the command row away, so the only way to act on something just found missing was also the broken way. **How it survived is the part worth keeping.** ⌘N and ⌘G reach both dialogs through the shell's shortcut handler without going near the palette, so both dialogs worked everywhere a person normally opens them; and `scripts/screenshots.mjs` photographs each overlay by **setting `overlay` directly**, so it holds a picture of a dialog that could not be opened by this route. That is **D-47's shape a second time** — a harness that constructs the state under test cannot see a transition that destroys it — and it is the second time in this phase that the thing which caught a defect was a person with the app in front of them | **Reordering the two calls** (`onclose()` first, then `command.run()`) — one line, fixes both rows, and leaves *ordering* as the thing that decides, so the next handler written in the obvious order is broken again with nothing to catch it. **Dropping `onclose()` from the command rows** and letting each command close what it replaces — correct for the two that navigate and wrong for the three that do not, so it trades a silent bug for a per-command rule nobody can see. **Stacking overlays** so the palette need not close — the prototype never stacks two and `Dialog.svelte`'s focus restore is written for one opener, which is the same dialog-over-dialog hazard row 12 is still waiting to test. **Making `overlay` a stack or a queue** — the general fix, and a state machine for a window that shows one overlay at a time. **Leaving it and re-wording row 11** — the palette is `MASTER.md` §7's "primary navigation surface" and two of its five commands did nothing |
| 2026-08-08 | **D-67** The sidebar becomes **one roving tab stop** — ↑/↓ move focus, Enter selects — rather than `docs/keyboard-audit.md` row 6 being re-worded to describe the thirteen tab stops that were actually there | The author reported "Tab does not get into Settings". Measured rather than argued: a throwaway script enumerated what the browser would treat as a tab stop, in document order, and the Settings pane **was** reachable — but at **stop 16 of 24**, behind **thirteen consecutive sidebar rows**. Fifteen Tab presses from the titlebar's own Settings button, which from a chair is indistinguishable from Tab never arriving. Row 6 had read *"Tab in, ↑/↓ between entries, Enter selects"* since it was written, and `Sidebar.svelte` had **no keydown handler at all** — so the middle clause was never implemented, and the row had already been ticked on the walk because Tab does enter and Enter does select. Thirteen stops is a legitimate nav list and the wording could have moved instead; what decided it is that the row is the one describing the *intended* keyboard shape, and a 13-stop sidebar is a cost paid on every single crossing of the window by the users this audit exists for. Re-measured after: **13 stops in the whole window, down from 24**, Settings four presses from the titlebar. **Arrows move without selecting**, unlike `Segmented`, because a radiogroup's value *is* its focused option while stepping through eleven views would re-filter the item list ten times on the way to the eleventh. Roles left alone — buttons in a `<nav>`, not `listbox`/`option` — because the global rules were measured against these roles and row 6 asks for none of it. Row 6 is **un-ticked** as a result: it passed against markup that no longer exists | **Re-wording row 6** to describe a plain nav list — no code changes, row 6 stays ticked, and it accepts fifteen presses forever on the reading that the wording was aspirational; rejected because nothing else in the row was aspirational and the cost lands on exactly the users S-08 is for. **Adding Home/End instead** — cheaper, and it helps only somebody who already knows the list is long. **Putting the footer's *Switch vault* in the same roving group** — row 6 names it in the same breath; kept a separate stop because it opens an overlay rather than selecting a view, and folding an action into a selection ring makes ↓ past Trash do something of a different kind. **Giving the group `listbox`/`option` roles** — the textbook shape for a roving widget, and a semantic change to a surface whose current roles are what the seven global rules were measured against. **Deferring until the walk finished** — offered and declined; the remaining rows are walked against this surface, so deciding after would have meant walking twice |
| 2026-08-08 | **D-68** Onboarding step 3 gets focus on its **first control** and an Enter handler **scoped to the step that ignores Enter on a `<button>`** — rather than focusing the acknowledgement checkbox, or handling Enter for the whole screen | Finding 4, the first defect the manual keyboard walk returned, and it is two defects sharing a screen. Steps 1 and 2 land focus with `autofocus` on their text field; step 3 has no text field, so it had nothing, and the password field leaving the DOM dropped focus to `document.body` — the first Tab restarted from the top of the *document* rather than from the kit on screen. And `onenter` is implemented on the input inside `TextField`, so a step with no text field had **no Enter path at all**: the row's "Enter finishes" was never implemented rather than broken, and it read as satisfied because the two steps before it satisfy it by accident of having an input. The Enter guard is the part that is not obvious: Enter already activates a focused button, so an unguarded handler would make Enter on *Print* print **and** finish — and `finish()` clears `recoveryCode` before routing, which would pull the code out from under a print dialog that is still holding it. This is also **finding 1 in a second place**: `Dialog.svelte` was fixed on 2026-08-07 for the identical "restored focus to nothing" failure, and the fix was made to the dialog rather than to the pattern, so the step transition kept it | **Focusing the acknowledgement checkbox** — it is the one thing the user must act on, and starting there puts Print and Save PDF *behind* them on the one screen whose content cannot be shown again. **A `<form>` with `onsubmit`** — the idiomatic Enter path, and it makes every button in the step a submit candidate on a screen where one of them opens the OS print dialog. **A global Enter handler on the onboarding screen** — one handler for all three steps, and it would fight the two `onenter`s already on steps 1 and 2. **Replacing all three steps' `autofocus` with one step-change effect** — the fix-the-pattern option and the tempting one; deferred because rows 1, 2 and 4 have not been walked yet and their `autofocus` is not known to be wrong, and changing a working surface on the way past is how a walk stops measuring what it started on. **Leaving it for the batch at the end of the walk** — which is what was planned on 2026-08-08 until row 3 turned out to block its own re-walk |
| 2026-08-08 | **D-69** `create_vault` is split: it creates the vault **in memory** and returns the kit, and a new **`commit_vault`** writes it and opens it. Onboarding step 3's acknowledgement is what calls the second one, so **nothing is on disk until the recovery kit has been acknowledged** | Found by the author while walking keyboard-audit row 3, and it is not a keyboard defect — it is a hole in R-07. The file was written at the end of step 2, `remember_vault` ran immediately, and step 3 then showed the kit. **A user who closed the window while reading the kit owned a vault whose kit had never been recorded**: R-07 shows it exactly once, `create_vault`'s own comment says there is no command to fetch it again, and the remembered path (D-40) sent the next launch to a lock screen. That vault has no recovery route for the rest of its life, and nothing in the product knew — the one requirement whose entire purpose is "what happens when the password is gone", defeated by closing a window. Two properties are now tests rather than intentions, both verified non-vacuous by breaking them on purpose: **a lock discards the pending vault** (it holds a decrypted key exactly like an open one, so onboarding must not be a way to keep a key alive behind a lock screen — the user-visible consequence, that the half-made vault is gone, is the right end for one whose kit was never written down), and **a failed commit puts the pending vault back**. The second is the defect the split itself introduced and it was caught before it shipped: the user now sits on step 3 for as long as it takes to copy 24 characters down, a directory can go away inside that window, and taking the pending vault out and dropping it would leave them holding the only rendering of a kit for a vault that exists nowhere, with a button answering `internal` from then on. The existence check is taken **twice** for the same reason — once at create so the user is sent back to step 1 before being shown a kit for a doomed path, once at commit because that is the check that actually guards the file. Budget unchanged at four: `commit_vault` returns nothing | **Leaving it and warning on step 3** that closing now means a vault with no recovery route — cheap, touches no requirement, and it moves the consequence onto the user, which is the false-promise pattern D-49, D-55 and D-61 were all deletions of. **Returning to the recovery-kit step on the next launch** — the author's first instinct and the one that cannot be built as stated: showing the kit again means storing something that unwraps the master key without the password, which is R-07's "exactly once" broken to fix R-07. The nearest legal version is *unlock with the password, then reissue* — a different feature, and the deferred write makes it unnecessary because there is no vault to be locked out of. **Writing the file and deleting it if onboarding is abandoned** — needs a reliable close hook, and a process killed between the two leaves exactly the vault this is about. **Keeping one command and passing an `acknowledged` flag** — the webview would then be asserting that the user has read something, which is a claim the host cannot check and the shape §9 check 6 exists to refuse |
| 2026-08-08 | **D-70** The design's **profile is made real** rather than left as the vault wearing a person's clothes: a `name` and an `email` go into the sealed body (`vault-format.md` §6.6), `vault_status` reports them, a vault-class `set_profile` writes them, and the sidebar footer gets the **popover the prototype always had**. *Sign out of TrustVault* and *Lifetime license · Manage* are deleted, not drawn inert | The author's report was that the footer had no dropdown and that Settings did not match the design. Both were true, and they are two different kinds of true. **The dropdown is a plain gap** — the prototype's footer opens a 216px popover with five rows and ours jumped straight into the vault switcher, so three real actions (switch vault, settings, lock) had no home and the row lied about where it went. **The content was a recorded deviation**, taken on 2026-08-04: with no account and no sync (D-03) an avatar, a name and an e-mail would have been three invented fields, so the footer and the Settings card carried the *vault* instead. That argument was correct and it settled the wrong question — it asked whether we could honestly *display* a person, when the option it never considered was to let the user **name one**. A profile that is stored is not invented. Four properties keep it from becoming an account: nothing authenticates against it, nothing validates it (an address with no `@` is legal — it labels a recovery kit, it does not receive mail), an untouched profile writes **no key at all** so every vault that predates this encodes to the bytes it did before, and it lives in the **sealed body** rather than beside the settings (D-33) because a name and an e-mail identify a person. The cost is stated rather than worked around: it cannot be read while locked, which is why `profile` is `null` there and the lock screen still names the vault. Two things fell out of the build that were not asked for and are not cosmetic. **⌘, is now bound**, because the popover prints it beside *Settings* and a shortcut drawn on screen that nothing listens for is D-61's defect at a tenth of the price. And the a11y audit returned a **real contrast finding on the footer avatar** — brass over `--accent-wash` over `--bg-hover` is 4.09:1, under §9's 4.5 — which is a defect that has existed since Phase 2 on the row's *hover* state and was invisible because `element.focus()` cannot hover; the new `.open` state is a state the audit can reach, so it found it. `--accent-chip` is the D-63-shaped fix | **Keeping the vault footer and only adding the dropdown** — the smallest change, honest, and it leaves the design's avatar meaning "the first two letters of a filename" forever. **Matching the prototype literally**, Sign out and Lifetime license included — pixel-exact and it puts two controls on screen with nothing behind them, which is D-49/D-55/D-61 for the fifth and sixth time. **Storing the profile beside the settings** (D-33's one plaintext store) — simpler, readable on the lock screen so the footer would never fall back, and it writes a person's name and e-mail address to an unencrypted file in a config directory, which is the one thing this product exists not to do. **Asking for it during onboarding** — it would mean no vault ever has an empty profile and no surface would need a fallback; rejected because the design's three steps are the three things a vault cannot be created without, and a fourth screen between a user and their vault to collect two optional labels is the wrong trade. **`get_profile` as its own command** — symmetrical with `set_profile`, and it is a second round trip for the same moment `item_count` already rides on. **`color-mix` for the avatar's opaque background** — no new token, and it computes to `color(srgb …)` which `scripts/audits/contrast.js` reads as `rgb(1, 1, 1)`: it turned one finding into eighteen false ones, which is the audit becoming useless rather than a cosmetic problem |
| 2026-08-14 | **D-71** A **third audit** joins `focus` and `contrast`: `scripts/audits/taborder.js` enumerates what the browser would treat as a tab stop, in document order, and fails four shapes — a roving group with no stop, a roving group with two, a positive `tabindex`, and a composite ARIA role (`menu`, `listbox`, `radiogroup`, `tablist`, `toolbar`, `tree`) holding anything other than exactly one stop. It **does not judge how many stops a surface has** | The throwaway script that found keyboard-audit findings 6 and 7 was deleted after it found them, which made the two most expensive defects of the phase findable only by someone remembering to write it again. The question it answers is the one neither existing audit can ask: both reach their elements with `element.focus()`, which succeeds on a `tabindex="-1"` control exactly as it does on a real tab stop, so a control that has left the tab order passes them both while being keyboard-unreachable. It paid on its first full sweep — **finding 10**, the profile popover built four days earlier with four tab stops where `role="menu"` promises one. What it deliberately refuses to judge is the other half of finding 7: thirteen consecutive sidebar rows is a defect against a row somebody wrote (D-67) and an ordinary navigation column anywhere else, and no property of the DOM separates them — so the count is *printed* (`--stops`) for a person to read and fails nothing. Two properties of the tool are stated rather than assumed: it measures **inside a modal** when one is open, because `Dialog.svelte` traps Tab, which assumes a trap it cannot itself verify; and it cannot press Tab, so focus traps and reading order stay manual rows. Verified non-vacuous by breaking `Segmented` on purpose in both directions — the historical finding-6 markup reproduces `unreachable-group`, a second stop reproduces `ambiguous-group` | **Leaving it as a throwaway** — free, and it is the option that already failed once: the script that found two defects was gone the next day. **Failing on a tab-stop count per surface, from a committed baseline** — it would have caught finding 7 mechanically, and it needs an expectation per surface that nobody has written and that changes whenever a tag is added to the sidebar, so the first regression it reports would be a fixture edit. **Making the roving rule structural rather than role-based** — the first version climbed while every operable descendant carried an explicit `tabindex`, which over-climbed out of `role="radiogroup"` and merged Settings' Theme with Interface size into one six-member group with two stops, and under-climbed in the sidebar and reported two of its three lists as unreachable: two false findings and one missed group on the same screen. **Requiring `inert` on the background instead of scoping to the modal** — it would make the DOM say what the JavaScript currently says, and it is an application change proposed by a measuring tool to make itself simpler |
| 2026-08-14 | **D-72** `npm run a11y` and `npm run shots` run in CI, on `browser-actions/setup-firefox`. The screenshots are **rendered and uploaded, not compared**: what fails the job is a screen that no longer renders, not a pixel that moved | Actions 12, 21 and 22 were one decision asked three times, and the answer was the same each time: a contrast failure is invisible in a diff and invisible in review, a control that leaves the tab order changes no pixel at all, and `npm run a11y` already exits non-zero — so the only thing between a token edit and a regression nobody sees was somebody remembering to run it. Eight minutes a run is the price, and it is bounded and known. **The screenshot half is deliberately weaker than it could be**: baselines taken on a developer's machine fail against a runner's font rendering on the first run, so a pixel comparison needs baselines produced by this runner image — a decision for the phase that has a release to protect, and a bad one to take by committing 50 PNGs today. What the render step does test is the class that has actually bitten: a scenario whose drive broke produces no file and fails the job, which is the same shape as the audits' "nothing to measure". Firefox comes from an action rather than apt because `ubuntu-latest` ships it as a snap transitional package | **`npm run a11y` only, leaving the screenshots local** — cheaper by half the runtime, and it leaves the twenty-five screens with nothing at all saying they still render. **Committing PNG baselines and comparing** — the real test, and it fails on font rendering on the first CI run rather than on a regression, which trains people to ignore it. **The mozillateam PPA with apt pinning** — no third-party action, and it is a third-party binary source either way with more moving parts. **A separate scheduled workflow rather than a PR job** — it keeps PRs fast and it reports the regression to nobody, a day later |
| 2026-08-15 | **D-73** Row 12 of `docs/keyboard-audit.md` is **re-worded to the surface that exists**: *Generate* fills the password field in place, reveals it, and leaves focus on it. The nested generator dialog its old clause described is **not** built | The clause read "the generator opens from inside it and returns focus", and nothing in the product does that — `NewItemDialog.svelte` mints 20 characters over all four sets, fills the field and reveals it (D-44). So the row could not pass and could not fairly fail: it described a surface nobody had built, which is the third time this audit has produced that shape after D-64 and D-67, and the third time the answer has been the author's rather than a silent edit. Re-wording rather than building is the smaller change and the safer one: a generator opened **from inside** the New-item dialog is a dialog over a dialog, and `Dialog.svelte`'s focus restore has already been wrong once about exactly that path (finding 1, fixed 2026-08-07). It also keeps D-44's one-generator rule intact — a second way to reach `generate_password` is a second thing to keep honest. The cost is stated rather than hidden: **length and character sets cannot be chosen while creating an item**, and the path for that is ⌘G plus the generator's own Copy button | **Building the nested dialog** — the row stands as written and the implementation catches up, which is D-67's answer to the same question; rejected here because D-67's clause described something the sidebar *should* do for a keyboard user, and this one describes a convenience with a working alternative one keystroke away. **Deleting the clause** — no decision to record, and the next person to read the row would not know a generator had ever been considered there. **Widening the fill-in-place button into a menu of lengths** — a third generator UI, and `MASTER.md` draws no such control |
| 2026-08-15 | **D-74** **Phase 4 opens with Phase 3's gate at 5 of 6.** The seven-day daily drive runs underneath it: the clock starts when the author's real vault is imported, the gate line ticks on its own day, and Phase 3 is not closed until it does | The drive is the only exit criterion in this project that costs **time rather than work** — seven days of using the application, during which nothing in the repository is blocked. Holding Phase 4 shut for it would buy a week of idleness and no evidence. The standard's rule is that skipping is allowed and skipping *silently* is not, so this is the record: the entry-check box "dependencies passed their gates" stays **unticked for the life of Phase 4**, which is what makes the deviation visible to whoever reads the phase document next. **The risk carried is real and is not the drive failing** — it is the drive returning a defect into a branch that has moved on. The further Phase 4 gets before day 7, the more of it a Phase 3 defect can invalidate, and the likeliest such defect is exactly the class the drive exists to find: something about living with the app across days, which no test here runs. Two things bound it — the drive runs on the **release binary** rather than on the development branch, so Phase 4's churn cannot disturb it, and anything it returns is a Phase 3 defect fixed on a Phase 3 branch rather than folded into Phase 4's work | **Waiving the drive entirely** — the gate line marked `not run — waived` and Phase 3 closed without it. Honest, permanent, and it retires the only test that asks whether the thing is livable; no later phase re-opens that question, and this project's own history says a person using the application finds what three passing audits do not. **Shortening it to three days** — catches the daily-rhythm defects (auto-lock mid-read, clipboard, reopening every morning) and not the weekly ones, and picking 3 over 7 has no argument behind it other than impatience. **Holding Phase 4 until day 7** — correct by the letter of the entry rule, and it spends a week of working time to remove a risk that two conditions above already bound. **Starting only the parts of Phase 4 that do not depend on Phase 3** — the shape the standard warns against by name: it moves the upstream risk into this phase while claiming not to have opened it |
| 2026-08-15 | **D-75** `feature/phase-4-watchtower` is cut from **`feature/phase-3-surfaces`**, not from `development` | The branch model says a phase branches from `development`, and `development` is 44 commits behind: Phase 3's gate is 5 of 6 (D-74), so none of the shell, the item commands, the settings or the surfaces Watchtower stands on are on it. A Phase 4 branch cut from `development` would be a Watchtower with no items to scan. This is the branch-level consequence of D-74 rather than a second decision — the phases overlap, so the branches do too — and it costs one thing worth writing down: **Phase 4's PR into `development` must not merge before Phase 3's**, or Phase 3's gate evidence arrives after the work it describes | **Merging Phase 3 into `development` now to unblock the cut** — tidy, and it closes a gate that has not passed by putting unproven work on the integration branch and leaving the Phase 3 PR with nothing left to carry. **Cherry-picking what Phase 4 needs** — an unbounded set that grows every time a Watchtower task touches a surface, and two copies of every Phase 3 commit at merge time. **Waiting for the drive** — the option D-74 already rejected, arriving one level down |
| 2026-08-15 | **D-76** Watchtower is **two** vault-class commands, not one: `watchtower_scan` (zxcvbn + reuse, no network) and `watchtower_breach_check` (HIBP, the only socket in the product). The setting is read in the **host**; neither command takes an egress argument | S-10 asks for **zero packets** when breach checking is off, and the two shapes make that claim differently. One command with a branch inside it makes "no egress" a line of code somebody must keep taking correctly, provable only by reading the function and re-reading it after every change. Two commands make it a **command nobody calls** — `tcpdump` on the app's PID then measures a property of the shape rather than of a branch, which is the same move that put the clipboard write in Rust instead of trusting the webview to forget a string. It also separates costs that have nothing to do with each other: the local half is milliseconds, repeatable, and the whole scan for a user who never opts in; the network half is minutes and depends on a service we do not run. The argument the split loses to is convenience — the view now makes two calls — and that is the cheaper half of the trade | **One `watchtower_scan(breach_check: bool)`** — one round trip and it puts the decision to touch the network in the webview, which is the layer the entire contract exists not to trust. **One command reading the setting internally** — the same egress guarantee, argued from a branch instead of from a shape; it is what this decision is a step away from. **A per-item `check_breach(item_id)`** — smallest command, and it moves a 1 000-request loop into the webview with the rate limiting, the backoff and the progress reporting spread across the boundary. **Rolling it into `list_items`** — scanning on every list draw is D-26's rejected option arriving a second time, now with a network call attached |
| 2026-08-15 | **D-77** Phase 4's two unmeasurable gate lines are **split rather than relaxed**, and the gate is now **seven lines instead of five**. Line 2 becomes *what leaves* — the capture shows a 5-character prefix and nothing else — and *what returns* — the padded response carries ≥ 1 zero-count row and is larger than the unpadded one for the same prefix. S-07 becomes **S-07a**, a wall-clock budget on the local scan with the network untouched and **its number left blank until the first measurement sets it**, and **S-07b**, the network half stated as behaviour: one request per distinct value, bounded concurrency, progress reported, an interrupted check leaving the vault consistent | Both original lines describe a service that does not behave that way, and neither could be met by writing better code. The row band is **impossible**: 1 924 of the rows are real hashes HIBP does not control, twice the criterion's ceiling before a single decoy is added, and the decoy count itself moved **110 → 125 → 156** for the same prefix inside one day — a criterion naming a row count cannot be held by a service that picks the number afresh per response. The ten seconds is **40× off**: 1 000 distinct passwords at the 1.87–2.5 req/s the live service gives is ≈ 400 s, and closing that gap needs 100 sustained requests per second aimed at a free API. What the re-wording preserves is the claim each line existed for — k-anonymity, padding actually on, and a scan that does not cost a request per item — stated as something measurable against a service we do not run. Splitting also puts the two halves where they can be measured: S-07a is a `cargo bench` line CI re-runs on every push, S-07b is read off a request log. **The number for S-07a is deliberately not named here**, per S-03's precedent — inventing one to fill the heading is what the standard forbids, and the first bench sets it | **Relaxing the numbers in place** — 800–1000 becomes "≥ 800 rows", ten seconds becomes ten minutes. Cheapest, and it is the failure the standard names: a criterion re-worded by the author of the code it measures until the code passes, with no record that the original ever said something else. **Deleting both lines** — honest about the fact that neither measures this code, and it retires the k-anonymity proof, which is the single claim R-25 exists to make. **Keeping them and marking them `not run — retrofitted` at G-C** — accurate bookkeeping for a gate nobody can pass, and it defers the argument to the phase that can least afford it. **Measuring the breach check against a mocked endpoint instead** — makes ten seconds achievable and measures the mock's loopback latency, which is a number about this laptop |
| 2026-08-15 | **D-78** zxcvbn scoring moves from `src-tauri/src/commands/strength.rs` into `crates/trustvault-core/src/watchtower.rs`. `score_password` becomes a pass-through, and the generator and `ipc_session.rs` call the core directly | The reason `strength.rs` was in the host is **not** the reason that decides this: it is there so the **webview** never parses 400 kB of dictionaries against S-01's 800 ms cold start, and that is untouched — the dictionaries were never shipped to the frontend and still are not. What decides between host and core is that scoring every password in the vault means **reading** every password in the vault. From `src-tauri` that means lifting a thousand plaintext values across a crate boundary, scoring them, and dropping them, when the crate that already owns every plaintext byte is one call away. The second reason is D-44's, one phase later: the score→word mapping and the crack-time channel were about to exist twice, and two definitions of "Weak" agree until the day one of them is edited. N-02 is unaffected and CI proves it — zxcvbn brings `fancy-regex`, `regex`, `itertools`, `lazy_static` and `time`, none of them on the boundary job's list. The cost written down rather than discovered: a Tier-1 crate gains that tree, and `time` is the crate D-18 raised the MSRV for | **Leaving the scoring in the host and calling it from Watchtower** — impossible in the direction it would need: the core cannot call the host, so it would mean the host pulling every password out of the core to score, which is the plaintext-spreading this architecture exists to prevent. **A copy in each** — the two-generators problem D-44 rejected by name, now with a threshold as well as a mapping to keep in sync. **A third crate for scoring alone** — a crate boundary, a manifest and a version for forty lines, and it would still have to be a core dependency to reach the vault |
| 2026-08-15 | **D-79** The weak threshold is **zxcvbn score ≤ 2**, not ≤ 1 | Set by measuring, and the first draft had it at 1. At zxcvbn's offline-slow-hashing rate: `password` 0 and `hunter2` 1 both die in **less than a second**, `Tr0ub4dour&3` scores **2** and falls in **31 minutes**, `Jakarta2019!` told the vault's own name scores **2** at **16 minutes**, and score 3 is 3 hours. The Watchtower view has read *"Crackable in a matter of hours"* under its Weak group since D-36 drew it from the prototype — **that sentence describes the 31-minute row, not the under-a-second ones**, so the design's own copy is evidence written before this decision existed. zxcvbn agrees from the other side: its documented meaning for 3 is the first score that resists an **offline** attack, and a breach corpus is offline by definition, so 2 buys protection from unthrottled online guessing and nothing more. A password manager that says nothing about a password crackable over lunch is not doing the job the screen claims | **≤ 1** — the meter's own "Weak" band, which is the assumption the first draft made silently. It leaves `Tr0ub4dour&3` unreported, and it makes the group's existing sentence false. **A length or character-class rule** — the thing the task forbids by name, and the measurement is why: twelve characters with four classes falls in 31 minutes while twenty-eight lower-case characters hold for centuries. **≤ 3** — catches the 3-hour row too, and it reports the majority of ordinary passwords, which is how a findings list becomes a screen nobody opens |
| 2026-08-15 | **D-80** S-07a's blank number is filled: **≤ 500 ms for a 1 000-item local scan**, set at 8× the first reading (61–63 ms, `cargo bench --bench watchtower`), and the benchmark **exits non-zero over it** as a step in CI's `rust` job | The rule D-77 wrote was that the first measurement sets the number the day it is read, and this is that day. The budget is not the reading, and the 8× is spent rather than left as slack nobody can audit: **×1.9** because the reference vault is zxcvbn's *cheap* case and the same run scans the audit fixture at 0.118 ms a password against 0.061 ms — a vault of passwords the dictionaries actually match costs about twice as much; **×2** for a machine that is not this desktop, the allowance every measured criterion here already carries; and the remainder is headroom to the point where a command that returns reads as a hang — the local scan reports **no** progress and cannot, because `watchtower-progress` exists for the breach half only, so half a second is the line and it is the same order as S-03's deliberate 511 ms unlock. Making the benchmark fail rather than print is what turns it from a number in a document into the criterion the gate line already claims it is: S-07a is the **only** measured criterion in this project a runner can re-take, since S-03 needs this desktop, the clipboard pair needs a display, and S-04's end-to-end half needs the running app. Two facts fell out of taking the measurement, both recorded in `trustvault-requirements.md` rather than left in a terminal: the budget is per **1 000 items** and a 10 000-item vault crosses it, and a scan of the reference vault returns **ten weak findings** — the single-digit indices, whose `pw-{index}-xK9` is eight characters — so anything asserting that vault scans clean asserts something false | **Setting the budget at the measurement** (say 75 ms) — the tightest criterion and the one that fails first on a runner or on a real vault, and it would fail for the *fixture's* reason rather than for a regression. **Naming a round 100 ms** — a number chosen rather than derived, which is what the blank existed to prevent. **Printing the number without a check**, as `kdf.rs` and `search.rs` do — right for those two, because neither can run anywhere but this desktop; wrong here, because this one can run everywhere and a criterion nothing enforces is a sentence. **Putting the budget in the CI workflow instead of the benchmark** — it would pass locally and fail only on a push, and the number would live where the person changing the scan never looks |
| 2026-08-16 | **D-81** The status cache the scan writes has one asymmetry: an item **with no password field** keeps the status it had — normally `unknown` — rather than being written `strong`. Scanned items with no finding are written `strong`, which is the only place "clean" is ever recorded | `Report` carries no row saying `strong` (§6.9), so absence from it is the only evidence a field is clean, and the caller has to write that. The question this decides is what the *absence of a password* means. A secure note, a Wi-Fi entry with only an SSID, an identity — Watchtower reads nothing on them, and `strong` is a **verdict**: the item list draws a green pip and the word *Strong* beside it, which would be the application claiming it checked something it never looked at. That is the same class as D-49's *Copy & autofill*, D-55 and D-61 — a true-looking statement with nothing behind it — and this one would appear on **every** note in the vault. The cost is real and is accepted rather than hidden: `ItemStatus` has one word for "never scanned" and none for "nothing to scan", so those items wear an `unknown` pip forever and the Watchtower screen must read `last_scan_at` (`vault-format.md` §6.7) to tell "this vault has never been scanned" from "these items have nothing to score". The timestamp exists partly for that | **Writing `strong` on everything the scan walked** — one fewer branch, and every note in the vault gets a green pip meaning "not applicable", which is the reading a user will never arrive at. **A sixth `ItemStatus`, `not_applicable`** — the honest vocabulary, and it is a format change plus a pip, a label, a filter and a stat tile in a design that draws five statuses; worth reconsidering if the Watchtower view turns out to need the distinction on screen rather than in a timestamp. **Deriving it in the view instead** — the view already knows an item's fields, so it could decide per row; it would put the rule in the surface where the item list and Watchtower would each need their own copy, and `list_items` elides the field values but not the kinds, so the two would drift the first time one was edited. **Leaving every item `unknown` and reporting only findings** — the simplest cache, and it makes the pip mean nothing at all: `strong` is what tells the user the scan reached an item and liked it |
| 2026-08-16 | **D-82** The Watchtower screen departs from the prototype's copy in two places, and both are deletions of a claim the product cannot make. The weak rows read **"Crackable in 31 minutes"** — zxcvbn's own phrasing, R-24 — where the prototype reads *"10 characters · word + year"*; and the lede says **nothing about Have I Been Pwned**, where the prototype says "hashes matched against Have I Been Pwned" | The character count is the one that matters, because it looks like nothing. **A character count is the length of a password**, and D-32 made every mask a fixed width — `"•".repeat(value.len())` was rejected by name — specifically so a secret's length never crosses this boundary. Printing it back in a findings list would undo that for exactly the passwords worth guessing: the weak ones, listed by item, on a screen a shoulder-surfer can read in one glance. What replaces it is not a substitute but the requirement — R-24 asks for the crack time **in words**, and `Finding::crack_time` already carries zxcvbn's phrasing, so the honest row is the shorter one. The HIBP clause is the D-49 pattern for the seventh time: breach checking is **off by default** (R-26), so on first run nothing has been matched against anything, and a sentence naming the service would be a claim about a request never made. The clause returns with the breach check, conditional on it having run. Two more lines on the same screen exist because of the same rule and are not decisions so much as consequences: **Expired and Breached draw no group** while nothing produces either verdict (§6.9), and a scan the host **refused** says so instead of rendering as a clean vault | **Reproducing the prototype exactly** — pixel-faithful, and it prints password lengths on screen and names a service that was never called. **Rendering length as a bucket** ("short", "very short") — coarser, and it still answers "how long is it" for an attacker who only needs the search space narrowed. **Deriving a note from the score alone** ("scored 2 of 4") — leaks nothing, and it is a number with no meaning to the person reading it, where "31 minutes" is the whole argument in two words. **Keeping the HIBP sentence and adding "when enabled"** — a sentence that describes a setting rather than what happened, on a line whose entire job is to say what happened |
| 2026-08-16 | **D-83** The HTTP client for the breach check is **ureq 3.4**, `default-features = false, features = ["rustls", "gzip"]`, with the bundled **webpki-roots** trust anchors rather than the OS trust store. It is confined to `src-tauri/src/hibp.rs`; N-02's CI job already named `ureq` among the crates the core may not reach | **The survey's first finding was that its own premise was wrong, and that is the reason to record it.** `Cargo.lock` carries `reqwest 0.13.4`, `hyper` and `tower`, which reads as reqwest being free — but `tauri` declares reqwest only under `cfg(any(target_os = "android", all(target_vendor = "apple", not(target_os = "macos"))))`, so on the three desktop triples the baseline has **no HTTP client at all**. `cargo tree --target all` says otherwise, and a decision taken on it would have been taken on a false premise. Measured per real target triple, crates added: **ureq +12/+12/+12** (linux/windows/macos, an identical set on all three) against **reqwest+rustls +30/+31/+35**. `cargo audit` is clean on every candidate lock. `gzip` costs **zero** additional crates — `flate2` is already under tauri — halves the transfer (80 MB → ~45 MB for the reference vault), and does not measurably narrow what `Add-Padding` masks: across five prefixes the padded size spread is **1.30× uncompressed and 1.31× gzipped**, so the padding's signal survives compression proportionally. Bundled roots over the OS store because verification is then **identical on all three platforms and reproducible**, and an interception CA installed in a user's OS trust store cannot silently sit in front of the range requests; the cost is that the root set moves on a dependency bump rather than with the OS, and `platform-verifier` is +2 crates the day that trade looks wrong. **The part of this decision with teeth is not the crate.** ureq's `Config::default` calls `Proxy::try_from_env`, defaults `https_only` to **false**, and follows up to **ten** redirects; all three are overridden in `hibp.rs` and each override is a test that fails when it is deleted | **reqwest 0.13 + rustls** — async-native so it matches Tauri's tokio runtime, HTTP/2, and by far the most-used client in Rust; rejected for 2.5× the crate delta on a workload §6.9 measured as bandwidth-bound (1.87 req/s serial, 2.5 req/s at 8 concurrent — HTTP/2 multiplexing buys nothing against 80 kB responses), and because its default `rustls` feature pulls **aws-lc-sys**, adding cmake and a C compiler to every CI job. **`tauri-plugin-http`** — +26 crates including **reqwest 0.12 beside tauri's 0.13**, a second major version of the same client in one binary, plus `cookie_store` and `publicsuffix`; and D-59's lesson that a plugin's cost is not only its tree. **attohttpc** — +12, the same delta as ureq, rejected on three counts: MPL-2.0 where everything else here is MIT/Apache, aws-lc-sys again, and **no connection pool at all**, which §6.9's connection-reuse requirement rules out by itself. **hyper directly** — full control, and it means writing pooling, redirect refusal and timeouts by hand in the one file in this application that opens a socket. **Native TLS** — schannel and Security.framework are free on Windows and macOS, and on Linux it is **openssl**, a system library on a project that ships a single binary |
| 2026-08-16 | **D-84** An egress rule is tested by watching the host the bytes would otherwise have gone to, never by reading the client's configuration back | Written as a decision because it was learnt by shipping the wrong version of it twice in one sitting. `hibp.rs` first carried `assert!(client.agent.config().proxy().is_none())`, and it **passed with the `.proxy(None)` override deleted** — `Proxy::try_from_env` returns `None` when the environment holds no proxy, so the test measured this machine's shell rather than this project's code. The redirect test had the same shape from the other direction: it accepted `Err(Http(302)) | Err(Offline)`, and with `max_redirects(5)` the followed request landed on a dead port, returned `Offline`, and satisfied the disjunction. Both now start a listener at the place the request must not reach and assert it is never contacted, and both were **verified non-vacuous by removing the override they guard**. The proxy one needs `ALL_PROXY` set in the process, so it lives in `tests/hibp_egress.rs` as its own binary rather than racing every other test that builds a client. This is the harness lesson of D-47 in the phase whose gate is a packet capture: a capture taken on a machine with no proxy set would have recorded a perfectly clean run either way | **Trusting the config assertion** — it is one line and it is what everyone writes; it cannot distinguish a set override from an empty environment. **Asserting only the returned error** — an error is compatible with the request having been sent, which is the whole question. **Waiting for the packet capture to catch it** — the capture is one run on one machine at the end of the phase, and it is the least reproducible evidence in the project |
| 2026-08-16 | **D-85** The breach check runs **four** range requests at a time, in a worker pool, and the number is a bound rather than a target | §6.9 measured this service from this machine: 48 cold prefixes serially is **1.87 req/s**, and eight concurrent is **2.5 req/s** — eight times the sockets for **1.34×** the throughput, because the rate belongs to Cloudflare rather than to us. So concurrency buys almost nothing and the only question left is how large a burst to impose on a free, unauthenticated API somebody else runs. Four is the smallest bound that still absorbs one stalled connection: a 30-second timeout on one worker leaves three working, where serial stops the whole check dead for those 30 seconds. It is a pool of OS threads rather than a semaphore over futures because D-83 chose a **blocking** client, and it is `std::thread::scope`, so the queries are borrowed rather than cloned into each worker — a `BreachQuery` holds a suffix nothing may copy carelessly | **Serial**, which S-07b's own wording permits — one stalled request stalls everything behind it, and the check is minutes long already. **Eight**, the configuration that was actually measured — twice the burst for a difference the user cannot perceive on a task that takes minutes either way. **A user-facing setting** — a number nobody can choose well, on a screen whose whole job is to say less rather than more |
| 2026-08-16 | **D-86** Only a **complete** breach check stamps `last_breach_check_at`. A pass with any value unchecked writes the breaches it found and leaves the timestamp exactly where it was | The timestamp is the only part of a breach check that survives the window: `unchecked` lives in the IPC response, and the next launch has nothing but the date. So a pass that reached three values out of a thousand and stamped *today* would have tomorrow's reader told this vault was checked — the stale-partial-pass-as-clean state §6.9 forbids in the same paragraph that defines the field. The hits are still written, because a breach found is a breach found and hiding it would be the error in the other direction. It also makes the vault's own record and the screen's record say different things on purpose: the screen reports *2 of 5 could not be checked* while the check is on screen, and the vault says *never checked* forever after | **Stamping on every run** — says "checked" for a run that checked nothing, and it is what the first draft did. **Stamping with a coverage fraction beside it** — a second field to keep honest, in a format §9 would rather not grow again, and it turns "when was this checked" into a number needing interpretation. **Requiring a person to re-run** rather than recording anything — the same answer with worse ergonomics, since the screen already offers *Check again* |
| 2026-08-16 | **D-87** Two more departures from the prototype's copy, both the same class as D-82. The breached row reads **"Found in 1,246 breach records"** where the prototype reads *"2024 data breach · 1.2M accounts"*; and the lede's closing clause switches to **"hash prefixes were checked against Have I Been Pwned — never a password"** the moment a request has been made | The prototype's row names an incident, a year and a victim count, and the range API returns **none of those** — it answers with an occurrence count and nothing else. Printing a breach we did not identify is the false-promise class of D-49, D-55 and D-61, so the row says the number it has. The lede is the same rule turned around: D-82 removed the mention of HIBP because with the check off nothing had been matched against anything, and once a check runs, *"nothing on this screen has left this device"* is the false sentence. It keys on a request having been made — the vault's timestamp **or** a check in this session — and the second half is not redundancy: a partial pass writes no timestamp (D-86) and still sent prefixes for everything it reached. **Found by rendering it**: the first draft read the timestamp alone, and the `watchtowerPartial` screenshot showed the old sentence directly above a row reporting what came back | **Keeping the prototype's row and filling it from the count** — "1,246 accounts" invents a breach event that the count is not evidence of. **Naming the corpus size instead** — a number about HIBP, on a row about this password. **Leaving the lede alone** — it was already true for every state that existed yesterday, which is exactly how a sentence becomes false without anyone editing it |
| 2026-08-16 | **D-88** The capture attributes packets to the app by sampling the sockets of its **process tree** every 200 ms, against a `tcpdump -i any` capture taken with **no BPF filter at all** | `tcpdump` has no notion of a PID, so "tcpdump on the app's PID" — which is how four gate lines are worded — has to be built out of something. Two things make this version worth writing down. **The tree, not the process**: a Tauri app on Linux is WebKitGTK, and WebKitGTK does its networking in a separate `WebKitNetworkProcess`, so attributing by the main PID alone would have watched the one process in the application least likely to open a socket and missed the one most likely to. **The unfiltered capture**: the report's claims are negatives — no password in the bytes, no host other than R-25's — and a negative is worth nothing if something was excluded before it was made. The cost is named rather than hidden: sampling can miss a socket that opens and closes inside 200 ms, so every check that can run against the *whole* capture does (the forbidden-string sweep and the hostname search), and attribution is used only to say which conversations were the app's. A missed sample can understate what the app did; it cannot hide a password | **A dedicated network namespace** — attribution exact by construction, and the GUI has to reach the display across it: X11's abstract socket is namespaced, `slirp4netns` is not installed on this machine, and a root veth with NAT changes the routing the capture is there to measure. Worth revisiting the day the app is captured headless. **nftables `socket cgroupv2` matching** into `nflog` — also exact, also root, and it is a firewall rule plus log plumbing for a run that happens twice. **`ss` polling with no capture** — tells you a socket existed, never what crossed it. **Capturing with a BPF host filter** for the HIBP address — cheap and small, and it answers "did it talk to HIBP" while making "did it talk to anyone else" unanswerable, which is the actual gate line |
| 2026-08-16 | **D-89** The capture proves gate line 2's **negative** and does not attempt its positive. That five characters were sent rather than forty is asserted by `hibp.rs` against a listener; what the wire evidence adds is that no password, no full hash, no title and no id appear in any byte, and that no host other than `api.pwnedpasswords.com` was named | The request is inside TLS, and no amount of capture reads it. This is written as a decision rather than a limitation because the tempting fixes are both worse than the gap. Making the plaintext readable means either a key-logging build — ureq exposes no rustls `KeyLog` hook, so it means patching the client, and shipping an application that can dump its own session keys is a hazard traded for a document — or a TLS-terminating proxy in front of it, which needs a CA in the trust store and then measures a client configured differently from the one that ships. Both replace the thing under test with something adjacent to it. So the capture confirms a wire and the unit tests confirm a request, and the report's last section says which is which, in the report rather than in a commit message | **Key-logging build** — reads the request exactly, and the artefact is a password manager that can be told to write its keys to a file. **mitmproxy with a local CA** — the standard answer, and it changes both endpoints of the connection being measured. **Inferring the request from ciphertext length** — a 5-character prefix and a 40-character hash are both a rounding error inside a TLS record, so the measurement cannot distinguish the two cases it exists to distinguish. **Asserting nothing and calling the gate line met by the unit test** — the gate asks for a capture, and the unit test cannot see a second connection to somewhere else |
| 2026-08-16 | **D-90** The capture runs against the **audit fixture** vault, not the reference vault and not the author's own | Three properties, and each rules out one of the alternatives. It holds `password`, which is genuinely in the corpus at 52 372 427 — so the live half of gate line 1 lands in the same run as the capture rather than needing a third. It holds **12 distinct values** across 21 password fields, which makes S-07b's "one range request per distinct value" a number small enough to count and different enough from the field count that a per-field bug is visible; the reference vault's thousand is ≈ 400 s of somebody else's free API to prove the same thing. And every string in it is a **published constant of this repository**, so the forbidden-string manifest is exact and generated rather than typed — the author's own vault cannot have its passwords written into a manifest, which is precisely why the capture cannot be run against it | **The reference vault** — already exists and already has a writer example, and nothing in it is breached, its passwords are derived so the sweep would be a regex, and a full check of it is minutes of network for evidence twelve requests give. **The author's own vault** — the most realistic, and the forbidden-string list would have to be typed from memory by the person least able to be objective about what they forgot. **A vault made for the capture alone** — a fourth fixture to keep in step with a format that is still moving; `auditfixture` already exists for R-23 and R-24 and its numbers are already re-read from zxcvbn on every test run |
| 2026-08-16 | **D-91** A forbidden string found **inside R-25's own hostname** is not a hit. The sweep masks every occurrence of `pwnedpasswords` before matching, and prints how many occurrences it excused | Found by running it: the first real `on` capture reported **twenty** hits and all **fifty-four** occurrences of `password` in the file were inside `api.pwnedpasswords.com`. The fixture's loudest password is a substring of the service's own domain name, which is a coincidence with teeth — the hostname is the one string the request is *required* to carry, and it appears in the DNS question, the DNS answer and the SNI of every connection. Masking rather than dropping the needle, because `password` is the value most worth searching for in every other byte of the capture; masking on the label `pwnedpasswords` rather than the full name, because DNS puts it on the wire as a length-prefixed label with no dots around it. The excused count is printed beside the verdict so the exemption is visible rather than silent — an exemption nobody can see is how a sweep quietly stops sweeping | **Dropping `password` from the needle list** — one line, and it blinds the sweep to the single most likely password to leak. **Renaming the fixture's password** — it would have to stop being `password`, which is the one value with a published breach count, a trimmed real response committed against it, and `5BAA6` as the prefix the whole fixture is built around. **Reporting the hits and letting a person judge** — twenty entries of noise on the report that decides a gate line, every run, forever; the second run is where somebody starts skimming them. **Masking the whole capture's DNS and TLS-handshake bytes** — broader, and it would also excuse a password that genuinely leaked into a DNS query, which is a real exfiltration shape |
| 2026-08-16 | **D-92** The focus audit measures **its own instrument** before it measures the application, and the a11y profile carries one pref — `datareporting.policy.dataSubmissionPolicyBypassNotification` — so that the page under test is in a window the browser considers focused | Run 31937787913 reported **1 556 findings, every one `no-indicator`** — 100 % of the focusable elements on every surface in both themes — and it was the browser. Reproduced on this machine and bisected to a single pref: on a fresh profile Mozilla's own build shows the data-collection privacy notice at startup, whatever presents it takes window activation from the content, and then `document.hasFocus()` is `false`, `:focus` matches nothing, and every ring in the app is invisible to `getComputedStyle` while `contrast` and `taborder` — which do not care who is focused — stay clean. Ubuntu's snap suppresses that notice, which is why this desktop was clean on the same commit and the same version, **153.0.4 both sides**: the audit had been measured for two days against the one build that hides the problem. So the pref is half the decision and the **probe** is the other half — `focus.js` now creates, styles and discards its own control, and reports separately whether `:focus` and `:focus-visible` paint on this browser. A failed probe is one honest failure of the audit rather than a finding against every control on the screen, and the run still exits non-zero. Neither half relaxes S-09: the ring must still come from the stylesheet. Verified both ways — the full 144-surface sweep is clean against Mozilla's tarball build with the pref, and making the probe's own rules unmatchable fails the run with the message that names the browser | **Widening the audit until it goes green** — the tempting one and the worst: an audit relaxed to pass on a runner is worth less than no audit, and `MASTER.md` §10's twelve boxes rest on this job. **`browser.display.show_focus_rings`** — would force a ring onto anything focused regardless of modality, so a genuinely ringless control would pass; rejected for exactly the reason the pref that *was* taken is acceptable. **`focusmanager.testmode`** — Gecko's own harness pref for an inactive window, tried first because the symptom looked like activation loss, and **measured to change nothing here**; dropped rather than carried as a charm. **Pinning the audit to Ubuntu's snap** — makes CI agree with this desktop by making CI stop being a second machine, which is what D-72 added the job to avoid. **Driving Firefox through Playwright or Marionette instead** — a second browser stack, and a heavier dependency than one line in a profile |
| 2026-09-09 | **D-93** The a11y harness uses a fresh Firefox profile per audit run, cleanup failure is not an audit result, and a timeout is named as a timeout | A targeted Watchtower tab-order run reached the browser and then failed in the harness twice: first `firefox.kill('SIGTERM')` threw `EACCES`, then the next invocation failed before opening the page because the reused `target/a11y-profile` directory was not empty. Both failures say nothing about the surface under test, and both are more likely after D-92 because the harness now has a real browser profile with prefs worth keeping isolated. A new profile under `target/a11y-profile-*` makes one stuck browser unable to poison the next run, and a denied cleanup signal is swallowed because the audit result is the page's posted report or an explicit timeout, not the operating system's willingness to signal a launcher process. The follow-up runs still timed out in this desktop's Firefox headless path, so the third fix changes the message: a page that never posted is now reported as a timeout rather than "the scenario drew no tab stop". Verified with `node --check scripts/a11y.mjs`; no two-theme browser result from this sitting is gate evidence | **Reusing and deleting one fixed profile** — tidy, and it lets a previous browser process turn the next audit into `ENOTEMPTY` before the app is measured. **Failing the audit on a denied cleanup signal** — loud, but it reports the browser launcher as an application defect. **Printing every timeout as zero tab stops** — strict-looking, but it lies about the failure mode. **Killing harder or cleaning with a wider command** — more destructive cleanup for a directory under `target/`, while the cheap answer is to stop sharing the directory |
| 2026-09-09 | **D-94** Phase 4's remaining manual evidence is accepted as already tested and recorded late | The Watchtower row 17 keyboard walk and the two packet-capture runs were not missing work; they were missing records. The user confirmed on 2026-09-09 that both had already been tested, so the project records the evidence rather than re-running heavy or privileged work in this sitting. This ticks row 17, the four capture-derived gate lines, S-07b, S-10, and the "capture evidence recorded" gate line. The risk is named in the wording: the evidence is user-reported and retrospective, in the same class as the earlier author observations that CI cannot reproduce | **Leaving the boxes open** — accurate only if the work had not happened, and it would make the project status wrong after the user clarified the evidence. **Re-running everything now** — stronger evidence, but it means a Tauri release build, `sudo`, live network capture and manual UI driving, which the user explicitly did not want in this session. **Marking them as machine-verified by this session** — false; this session recorded the already-run tests |
| 2026-09-09 | **D-94** Phase 4's remaining manual evidence is accepted as already tested and recorded late | The Watchtower row 17 keyboard walk and the two packet-capture runs were not missing work; they were missing records. The user confirmed on 2026-09-09 that both had already been tested, so the project records the evidence rather than re-running heavy or privileged work in this sitting. This ticks row 17, the four capture-derived gate lines, S-07b, S-10, and the "capture evidence recorded" gate line. The risk is named in the wording: the evidence is user-reported and retrospective, in the same class as the earlier author observations that CI cannot reproduce | **Leaving the boxes open** — accurate only if the work had not happened, and it would make the project status wrong after the user clarified the evidence. **Re-running everything now** — stronger evidence, but it means a Tauri release build, `sudo`, live network capture and manual UI driving, which the user explicitly did not want in this session. **Marking them as machine-verified by this session** — false; this session recorded the already-run tests |
| 2026-08-02 | **D-14** G-A dropped, G-B replaced by G-B′ | Software-only project: there is no pin mapping and no board bring-up. The IPC security boundary is the structural equivalent of "the physical thing behaves as drawn" | Keeping the hardware gates as empty ticks — which would make "already checked" indistinguishable from "never considered" |

## Open questions

- [x] **Phase 4's gate line 2 is not measurable as written** — raised 2026-08-15 at the entry
      check, **closed the same day by the author, D-77.** The proposal below was taken as written:
      the line is now two, *what leaves* and *what returns*.

      **What acting on it cost, and it is D-64's lesson holding a second time.** The proposal was
      written from the same reading that produced the wrong line, so it was re-measured against the
      live service before being written down — and the re-measurement returned a **third** decoy
      count, **156**, over the same 1 924 real rows. That is not a correction to the proposal; it is
      the proposal's own argument arriving a third time, and it is now in R-25 as three numbers
      rather than one. One implementation hazard came out of the same fetch and is worth more than
      the count: the response is **CRLF-terminated and its last line carries no terminator**, so a
      naive `split('\n')` leaves `\r` on every row — the suffix still matches and the count parse is
      what breaks. It is written into §6.9 rather than left for `hibp.rs` to rediscover.

      The question as it was raised: it asks packet capture to confirm that HIBP responses "contain 800–1000 rows
      (padding honoured)". Measured against the live service the same day, prefix `21BD1`
      returned **2034 rows with `Add-Padding: true` and 1924 without**, the difference being
      **110 zero-count decoys**; the 1924 real suffixes are byte-identical in both. The band is
      not merely unmet, it is **impossible**: the real hashes alone are twice its ceiling, and
      they are data HIBP does not control. The line describes padding *to* a fixed size; the
      service adds a variable number of decoys *on top of* a real count.

      **The proposal, for the author to take or replace.** Split it in two, because the
      requirement is really two claims. *What leaves the machine*: the capture shows a
      **5-character prefix and nothing else** — no full hash, no password, no item title — which
      is the k-anonymity claim and the one R-25 exists for. *What comes back*: the response
      carries **at least one zero-count row**, and the response with the header is **larger than
      the response without it** for the same prefix, which is padding honoured, stated as
      something the service can actually be held to. Both are checkable in one capture.

      Left as a question rather than edited in place, because a gate criterion that the code's
      author re-words to fit what the code found is not a gate — the same reason D-64, D-67 and
      D-73 were the author's calls.

- [x] **S-07 is not measurable as written** — raised 2026-08-15 while writing the contract,
      **closed the same day by the author, D-77.** The proposal below was taken: S-07a is the local
      half with **its number left blank until the first bench sets it**, S-07b is the network half
      stated as behaviour rather than as a clock.

      **And acting on it found the thing the proposal had not looked at: the fixture.** S-07a is
      measured against the reference vault, and `benchfixture.rs` gives every one of its 1 000 items
      the password `pw-{index}-xK9`. No two items share a value, so **R-23 has nothing to group**,
      and every value is the same short unmatched shape, so **R-24 has one score repeated a thousand
      times** — zxcvbn spends its time on dictionary matching and this fixture hands it none. The
      vault is a fine timing **floor** and it is not a fixture the two scoring requirements can be
      measured against at all. Two tasks came out of it rather than one, and the caveat travels with
      S-07a's number so the first person to read it does not mistake the cheap case for the typical
      one. The question as it was raised:

      Raised before a line of Watchtower code existed, which is the whole argument for writing the
      contract first. It asks for a full Watchtower scan of the reference vault, **including HIBP**,
      in ≤ 10 s. The reference vault is 1 000 items with 1 000 **distinct** passwords
      (`benchfixture.rs`), so a full breach check is 1 000 range requests — one per distinct value
      is already the floor, not the naive implementation.

      **Measured against the live service, same day, from this machine.** 48 cold prefixes over one
      reused connection: 25.6 s, **1.87 req/s**. The same 48 at concurrency 8: 19.0 s, **2.5
      req/s** — a 2× improvement for 8× the parallelism, so the limit is not the handshake. At that
      rate 1 000 requests is **≈ 400 s**, and about **80 MB** of response body. Ten seconds buys
      roughly **25** distinct passwords. Hitting the criterion needs 100 requests per second
      sustained, which is not achievable from here and is not a polite thing to aim at a free
      service. Two smaller measurements came with it: padding costs **+6.4 % of bytes** (75,622 →
      80,497 for `21BD1`), so R-25 has no size trade to argue about; and the decoy count is **not
      fixed** — 110 at the entry check, 125 hours later, same prefix, same real 1,924 rows. That
      last one is independent confirmation for gate line 2's re-wording above: a criterion naming a
      row count cannot be met by a service that picks the number afresh per response.

      **The proposal, for the author to take or replace.** Split it, because it is two criteria
      wearing one number. *The half this code owns*: `watchtower_scan` — zxcvbn and reuse over the
      reference vault, **network untouched** — completes within a wall-clock budget, measurable in
      CI on every run with no service involved. The number should be **set from the first
      measurement** rather than guessed here, the way S-03's 511 ms was; naming one now would be
      inventing a value to fill a heading. *The half the network owns*: state it as what the code
      can be held to rather than as a clock — **one request per distinct value**, bounded
      concurrency, progress reported, and an interrupted check leaving the vault consistent. A
      wall-clock number for the network half measures the user's link and HIBP's cache, not
      TrustVault.

      Left as a question rather than edited in place, for the reason the line above it is: a gate
      criterion re-worded by the author of the code it measures is not a gate. Fourth time in this
      phase's first day that taking a criterion seriously has falsified it — D-64, D-67, D-73, the
      entry check, and now this.

- [x] **Import from other password managers** — **closed 2026-08-05, D-42.** One format:
      Bitwarden's unencrypted JSON export. R-29 is now a testable requirement with six Phase 3
      tasks, and the four paths not taken are out-of-scope lines in `trustvault-project.md` rather
      than silence. The survey's transferable finding is in the acceptance criterion, not the
      format choice: every importer surveyed fails by **dropping a field in silence**, so R-29 is
      met only when every field is either mapped or named in a refusal.
- [x] **Audit-log retention and location** — **closed 2026-08-04, D-31.** Inside the sealed body as
      a new `audit` key, which §9 of the format spec already permits without a version bump; flushed
      on save rather than on every reveal; capped at 1000 entries; setting off by default. No prior
      art was found to copy — 1Password's is server-side and Business-only, KeePassXC has none — so
      the conservative default was chosen deliberately rather than by convention.
- [x] **Argon2id default parameters** — **closed 2026-08-02.** 22 parameter sets timed on the dev
      machine; 256 MiB / t=3 / p=1 gives 511 ms, clearing S-03's 500 ms floor by 2 %. The thin
      margin is the finding: the answer to "per platform" is not a constant per operating system —
      the variable is the machine, and a 2019 laptop on Linux is slower than a 2025 laptop on
      Windows. `KdfParams::calibrate` measures the machine at vault creation and stores the result
      in the header, so the constant is only a starting point. Algorithm and full table:
      `docs/vault-format.md` §3.2.
- [x] **Where do settings live?** — **closed 2026-08-04, D-33.** Plain JSON in the OS app-config
      directory, all four values including `audit_log_enabled`. The log stays sealed; the switch
      governing it does not need to be, and splitting four non-secret values across two stores to
      keep one boolean company would buy two formats and two migration stories.
- [x] **R-10's acceptance criterion is unmeetable as written** — **closed 2026-08-05, D-39.**
      Corrected to name `src-tauri/tests/ipc_audit.rs`, with the reason (N-02 forbids the core from
      seeing a command) written into the requirement so the correction cannot later read as a
      weakening.
- [x] **The Phase 2 gate asks to "read an item", and Phase 2 cannot create one.** — **closed
      2026-08-05, D-38.** Re-worded to what Phase 2 can demonstrate; "read an item" is now the
      Phase 3 gate's first line, where `add_item` exists. The risk that comes with it — G-B′
      passing without a human watching a secret cross IPC in the running app — is written into
      the phase document rather than absorbed.
- [x] **Does the "Clears in 12s" chip survive contact with clipboard managers?** — **closed
      2026-08-05, D-41. It does not.** GPaste 45.3 on GNOME/Wayland recorded the value with the
      `x-kde-passwordManagerHint` set and still held it after TrustVault cleared the clipboard.
      Two findings for the price of one: the hint was **not being sent at all** until this test
      was written, which is the entire justification D-11 chose `arboard` on. It is sent now, and
      it is ignored. UI copy changed to "TrustVault clears its copy in *n*s".

- [ ] **The meter says *Fair* and Watchtower says *Weak* about the same password** — raised
      2026-08-15 by D-79, and it is a copy decision rather than a threshold one. zxcvbn score 2
      renders as **Fair** on `StrengthMeter.svelte` (the mapping predates this phase, and it is
      the prototype's) and lands in the **Weak passwords** group in Watchtower. Both are
      defensible on their own: the meter is scoring a password the user is *typing*, where
      "Fair" is encouragement to keep going, and Watchtower is scoring a password already in
      use, where the question is whether to change it. Said aloud, though, it is one number
      called two words on two screens of the same application, and the user meets both.
      Three shapes to choose between, and **not** by editing whichever file is open: rename the
      meter's band at 2 so the words match; leave the meter and re-word the Watchtower group so
      it does not claim the word "weak" for a *Fair* password; or state the split deliberately in
      both surfaces' copy — "fair to type, not fair to keep". Left as a question because
      `MASTER.md` §2 makes status wording binding rather than incidental, and because a
      re-wording chosen by whoever noticed the collision is how two screens end up disagreeing
      a third way.
- [ ] **`list_tags` is specified and nobody needs it** — raised 2026-08-06 while implementing
      `search_items` beside it in §6.5. The contract justifies it as "for the sidebar's tag list
      and the Add dialog's chips", and both of those already build their lists *and their counts*
      from `list_items`: `ItemSummary.tags` carries every tag to the frontend, which §6.1 permits.
      Implementing it would add a second path to data the webview already holds legitimately, so
      it is left `// planned` rather than shipped or deleted — removing a command from the
      contract is a decision, not a tidy-up. What would change the answer is a count the list
      cannot compute, which is what a vault too large to list would produce.
- [x] **The Phase 3 gate's R-29 line cannot be met as written** — raised 2026-08-06 while building
      the importer, **closed 2026-08-08 by the author, D-64.** The re-wording was taken: *all of
      **Bitwarden's** item types, producing the five of ours they map onto*. What the answer cost
      was not the edit — it was that acting on it meant asserting the fixture actually covered
      them, and it **covered six of the eight**. The proposal itself said "all seven of
      Bitwarden's", carrying our own number across into a sentence about theirs, and 7 and 8 —
      the driving licence and the passport `import/bitwarden.rs` names in its own comment — were
      in no fixture and no test. Both are in it now and the count is a test. The transferable part
      is that a re-wording proposed on one day and taken on another is **not** a paperwork edit:
      the proposal was written from the same memory that produced the wrong line, and nothing
      re-checks it until somebody tries to tick the box.
- [x] **Import has no surface, and the app has no file picker** — **closed 2026-08-06, D-59.**
      Both halves landed together, which is what the question predicted: the survey answered two
      Phase 3 tasks, and import lives in Settings with `MASTER.md` as the only authority, as in
      D-48. `tauri-plugin-dialog` was adopted and the webview granted **nothing** — the picker is
      a host command returning a path, and the capability file is still `core:default` alone,
      asserted rather than described. The finding worth carrying past this row is the one nobody
      was looking for: the plugin **rewrites `window.alert` and `window.confirm`** in our page,
      and the replacement `confirm` is async, so the ordinary way to write a confirmation would
      confirm itself. A dependency's cost is not only its tree.
- [ ] **Nothing in the webview resets the auto-lock clock** — raised 2026-08-06 while wiring
      `copy_generated`, and now the oldest open behavioural question in the phase. Only
      vault-class commands `touch()`, through `with_vault`, so a user filling in the New-item
      dialog for longer than `auto_lock_seconds` is locked out mid-form and the form's contents
      die with it. **The import surface widens it**: reading a preview's refusal list is exactly
      the kind of long, deliberate reading this app does not count as activity, and `import_commit`
      then answers `locked` after the user has already been shown what would be imported. Two
      shapes to choose between — an ambient `touch`/heartbeat command the webview calls on real
      interaction, or activity tracked in the host window itself. Do **not** answer it by
      extending the timeout. It is listed here rather than as a task because it is a decision
      first; the 7-day drive is what will say how badly it bites.

## Next actions

1. ~~Run the functional gate line.~~ **Done 2026-08-05 by the author** — created, quit, relaunched
   onto the lock screen, unlocked. G-B′ passed, Phase 2 closed.
2. ~~Settle the item model before any command is written.~~ **Done 2026-08-05, D-43.**
3. ~~Extend `docs/ipc-contract.md` before the commands, not after~~, and ~~decide
   `generate_password`~~. **Both done 2026-08-05** — D-44, with D-45 and D-46 falling out of the
   same sitting. The contract now specifies fifteen unimplemented commands, each marked
   `// planned`, and the harness enforces the marker in both directions.
4. ~~Write `add_item`, `update_item`, `delete_item` next~~ **Done 2026-08-06.** Both details the
   note flagged as silent-when-wrong are pinned by tests rather than by care: `value: null` meaning
   *unchanged* has a unit test and a session-harness assertion, and the harness was verified
   non-vacuous by breaking the rule on purpose. ~~The importer is still owed~~ — **its core and
   both commands landed the same day**; what stands between today and the 7-day clock is now the
   import *surface*, and that needs the file-picker decision in the open questions above.
5. ~~Wire the three commands to their surfaces.~~ **Done 2026-08-06.** All three go through
   `src/lib/ipc.ts` beside `copyField`, none through a stub. It cost two decisions the task list
   had not anticipated — **D-47**, the argument-case bug that had made three commands unreachable
   since Phase 2, and **D-48**, the edit dialog, which the prototype does not draw — plus **D-49**,
   a false promise in the delete copy.
6. ~~Build the generator.~~ **Done 2026-08-06 (later)** — all four boxes, `generate_password` and
   `copy_generated` shipped with their markers deleted, `src/lib/passwords.ts` deleted, D-52 for the
   two departures from the prototype. Two things it cost that the task list had not anticipated, both
   cheap: the screenshot harness needed to answer the new commands with a **fixed** password (a shot
   that differs on every run cannot become the baseline action 12 asks for), and `npm run fmt:check`
   was **already failing on `development`'s newest file** — the Bitwarden fixture, one line over
   width, committed 2026-08-06. Reformatted rather than added to `.prettierignore`, because a
   prettified export is closer to what Bitwarden actually emits, not further from it.
7. ~~Nothing the user does in the webview resets the auto-lock clock.~~ **Moved into the open
   questions above on 2026-08-06**, unchanged and unanswered, because it is a decision rather than
   a task. The import surface widened it: reading a refusal list is a long, deliberate read that
   the app does not count as activity, and `import_commit` then answers `locked` after the user has
   been shown what would be imported.
8. ~~Run the first gate line by hand, now that it is possible.~~ **Done 2026-08-15** — item
   created through the UI on the author's own vault, quit, relaunched, unlocked, read back, on
   `target/release/trustvault` built from `be278bc`. The gate line ticks. It stays an observation
   rather than a measurement, and nothing in CI reproduces it. The original note:

   **Run the first gate line by hand, now that it is possible.** *Create an item through the UI,
   quit, relaunch, unlock, and read it back — on Linux, against a real file on disk.* This is the
   line D-38 moved out of G-B′ into this gate, and until today nothing could satisfy it. It is
   also the only check that would have caught D-47, so running it is worth more than usual:
   `ipc_session.rs` passes and passed all along, and the app did not work. **Nobody has yet seen
   a secret cross IPC in the running app.**
9. ~~A tag can be chosen but never created.~~ **Done 2026-08-06 (later still)** — a chip-shaped
   *New tag* field in both dialogs, folding a case-only difference onto the tag that exists, and
   swallowing Enter so adding a tag cannot submit a half-filled form. The Tags control is drawn
   unconditionally now; it used to be hidden on a vault with no tags, which is the one vault that
   needs it.
10. **Delete a `// planned` marker in the same commit as the command it describes.** Held every
    time it has come up — the first three, the generator's two, `search_items`, the TOTP pair, and
    the four vault commands. **One marker remains**, `list_tags`, and it is the one that is
    deliberately unimplemented: `ItemSummary.tags` already carries every tag to the frontend, so
    implementing it would add a second path to data the webview holds legitimately. It stays
    specified and unregistered until something needs a count the list cannot compute, which is an
    open question above rather than a task.
11. ~~Carry two settings rows into Phase 3.~~ **Done 2026-08-06** — all five fields, both surfaces,
    and both new behaviours. Two things it cost that this note had not anticipated. `tokens.css`
    "already implements the scaling half through `[data-ui-scale]`" is true and the sentence above
    it was not: the mechanism is a **multiplier, not root `rem` scaling**, which `MASTER.md` §3 and
    the contract both claimed — **D-57**, with a CI grep holding the condition that makes the two
    equivalent. And `merge_incoming` had to grow from one host-owned field to **four**, because
    window geometry is measured by the host on every resize while the webview holds a `Settings`
    from when its screen opened.
12. ~~Consider wiring the screenshot harness into CI.~~ **Answered 2026-08-14 as D-72, together
    with actions 21 and 22 — they were one decision asked three times.** Both runs are a CI job
    now. The screenshots are **rendered and uploaded, not compared**: a pixel baseline taken on
    this desktop fails against a runner's font rendering on its first run rather than on a
    regression, so what the step tests is that every screen still renders — the class that has
    actually bitten, and the same shape as the audits' "nothing to measure". A real comparison
    needs baselines produced by the runner image and belongs to the phase with a release to
    protect. The original note is kept below, because the reasoning it carries is why the answer
    came out this way.

    `scripts/screenshots.mjs` now renders
    **twenty-five** screens in both themes without a Tauri host — the edit dialog, the delete
    confirmation, the Add dialog's filled-in 2FA seed, and then an empty vault, Trash and a
    palette query matching nothing were added 2026-08-06 — so a visual regression is checkable by
    something other than eyes. It gained a `fill()` driver with the 2FA shot, because a surface
    that only appears once a field has something in it cannot be photographed by clicking alone,
    and a fixture override with the empty ones, because an empty list is a different fixture
    rather than a flag on the same one. The three newest are the argument for the whole tool:
    **Trash is empty by construction in v1**, so its copy is on a screen nobody visits, and that
    is precisely where a false sentence sat undisturbed for a day. What it needs to be a *test* rather than a tool is a baseline and a comparison;
    that is a Phase 3 call, and worth making **before** eight more surfaces get wired rather than
    after. Note what it would **not** have caught: it stubs `invoke` itself, so D-47 was invisible
    to it too.
13. ~~Answer the two open questions the importer raised.~~ **Both answered.** The expensive one is
    **D-59**, and it did answer both tasks as predicted: the import surface exists and *Open vault
    file…* works, from one survey and one dependency. The cheap one — the gate's "all seven item
    types", which no export of Bitwarden's can produce — is **D-64, taken by the author
    2026-08-08**. It was the cheap one only until somebody acted on it: the re-wording's own
    number was wrong (seven, where Bitwarden has eight) and the fixture it licensed covered six of
    the eight. Both facts are tests now rather than sentences.
14. **Remember what CI does not reproduce.** `clipboard_clear.rs` prints SKIPPED on a runner with
    no display and `clipboard_manager.rs` is `#[ignore]`d, so neither clipboard measurement in
    G-B′'s evidence comes from anywhere but this desktop — and now neither does the functional
    line. **D-47 widens this**: nothing in CI runs the app, so nothing in CI can see a command
    that is registered, documented, unit-tested, and unreachable.

    **`cargo bench` was in the same position until 2026-08-15 and nothing said so.** No workflow
    ran any of the three benchmarks; S-03's 511 ms and S-04's 0.85 ms p95 are readings from this
    desktop that no push re-takes, and the S-07a gate line was written claiming CI re-runs it.
    Only S-07a is a CI step now (D-80), and only because it is the one benchmark a runner can
    take honestly: Argon2id's work factor is a statement about **this** machine's calibration, and
    a runner re-taking it would report a different machine's number under the same criterion ID.
    The two that stay manual now say they are manual in the criteria table rather than being
    assumed automated.
15. **Take S-04's end-to-end measurement in the app.** The host half is done and was never where
    the risk sat — 0.85 ms p95 of a 50 ms budget. What is unmeasured is the IPC hop and the
    render, and the instrument is already in place: `CommandPalette.svelte` emits
    `palette-keystroke-to-render`, readable from the devtools console with
    `performance.getEntriesByName(…)`. It needs the reference vault *in the app*, which means
    `benchfixture` writing a `.tvault` a build can open — the fixture exists now, the file does
    not. Worth doing in the same sitting as action 8: both want the app running against a real
    file on disk, and neither can be run by CI.

    **Take it from the `measure` build, not from `tauri dev` (D-65, 2026-08-08.)** The devtools
    console is only in a debug build, and a debug build's matching is 7.8× slower than the one
    that ships — the reading would have been 12 % budget-inflated and indistinguishable from a
    real one. `npm run tauri build -- --no-bundle --features measure` is the shipping profile with
    the inspector turned on, and it opens the inspector itself so the console is not a hunt.
    Sample size is the thing to watch while typing: the palette **drops** a keystroke's measure if
    a later keystroke supersedes it before its results render, so fast typing produces fewer
    samples than keystrokes and the ones it drops are the slow ones. Type deliberately, and read
    `length` off the entry list before trusting the percentile.
16. **`launch_at_login` works on Linux and is written-but-unverified on macOS and Windows.** The
    round-trip test runs on the CI platform only — `#[ignore]`d elsewhere, and it writes into the
    real user profile, which is why. Nothing is wrong with the other two paths as far as reading
    them goes; nothing has run them either. It belongs in G-C's per-platform re-measurement rather
    than in this phase, and it is listed here so it is not discovered there.
17. **`switch_vault` does not check the file is still there.** Switching to a vault deleted or
    unmounted outside TrustVault lands on a lock screen that cannot unlock. It is recoverable —
    the switcher's **Leave** removes the row — so it is a rough edge rather than a trap. The
    tempting fix is an `is_file` check in `switch_vault`, which is **wrong** for `restore_last_vault`'s
    reason: it would make the switcher silently refuse a vault on a drive that is merely unplugged.
    The right fix is the switcher knowing which files exist, which is a second command and a
    decision.
18. ~~What is left in Phase 3 is six tasks~~ — ~~three, as of 2026-08-07~~ — **two, as of
    2026-08-08**, and both want the app in front of a person. Three of the four accessibility
    boxes closed on 2026-08-07: the focus ring and the contrast audit are measured by
    `npm run a11y`, and the D-36 sweep found and resolved four stale disabled controls (D-60 to
    D-62). **R-29's wording closed 2026-08-08 as D-64**, and with it the import group at 6 of 6
    and the gate's R-29 line. What remains: the keyboard audit's nineteen surfaces, and S-04's
    end-to-end number.

    **The critical path is the author's, not the code's**, and it has been since the import
    surface landed. The 7-day clock needs the author's own passwords in a vault. Actions 8, 15,
    19 and 21 are one sitting, and nothing in CI can do any of them.

    **The sitting, in the order that makes it one sitting rather than three.** It takes **two**
    binaries of the same commit, and that is not a workaround: the `measure` build opens an
    inspector into the window (D-65), which is a pane the keyboard walk would have to Tab through
    and a build no user runs, so the nineteen rows and the functional line belong on the default
    one. ~~Both are built and waiting, along with the reference vault~~ — **all three are stale as
    of 2026-08-14 and rebuilding them is step zero of the sitting.** The two binaries were built
    at 12:11 on 2026-08-08 and the profile landed at 18:13 the same day (D-70), so they predate
    **rows 20 and 21 entirely** and also predate D-69's deferred vault write, which changes what
    row 3 exercises. Walking them would walk an application four commits old and report rows that
    do not exist in it. `/tmp/reference.tvault` is gone, as `/tmp` promised it would be. Rebuild
    the `measure` one **first** and copy it aside, because the default build overwrites the same
    path:

    | For | Binary | Built |
    |---|---|---|
    | All 22 rows, and the functional line | `target/release/trustvault` | `npm run tauri build -- --no-bundle` — **second**, after the one below has been copied aside |
    | S-04 only | `target/release/trustvault-measure` | **first**: `npm run tauri build -- --no-bundle --features measure`, then `cp target/release/trustvault target/release/trustvault-measure` before the default build overwrites the path |
    | The vault S-04 is measured against | `/tmp/reference.tvault` | `cargo run --release --example reference-vault -p trustvault-core --features benchfixture -- /tmp/reference.tvault` — password `correct horse battery staple`, and `/tmp` means regenerate it if the machine has rebooted |

    Then: unplug the mouse **before** launching `trustvault`, and walk
    `docs/keyboard-audit.md`'s twenty-one rows (action 21) — recording as you go rather than at the
    end, which is that document's own rule. Row 19 needs the reference vault open, because
    *Choose file…* has to hand focus to the OS and get it back. Then run the functional line
    (action 8) on your **own** vault rather than the reference one, because that is the vault the
    seven days start on. Plug the mouse back in, quit, and launch `trustvault-measure` last for
    S-04 alone: open `/tmp/reference.tvault`, type into ⌘K, and read
    `performance.getEntriesByName('palette-keystroke-to-render')` — checking `.length` against how
    many characters you typed before trusting the percentile, since a superseded keystroke is
    dropped rather than timed.
19. **Run the import against a real export before trusting any of it.** Every assertion behind
    the importer is against `tests/fixtures/bitwarden-export.json`, which this project wrote —
    so it tests the parser against our own reading of the schema, not against what Bitwarden
    emits today. The three things a real file will decide: whether the refusal list is short
    enough to read or long enough that nobody does, whether `otpauth://` seeds actually arrive in
    the wild in the shape `import/bitwarden.rs` handles, and whether the auto-lock clock expires
    while the preview is on screen (the open question above, in the place it will first bite).
20. **The picker is unexercised on macOS and Windows**, exactly like `launch_at_login` (action
    16), and for a sharper reason: `rfd`'s macOS path needs an `NSApplication` for a truly async
    dialog and falls back to a **sync** one without it, which from an `async fn` command is the
    difference between a dialog and a frozen window. Nothing here is suspected — nothing has run
    it either. It belongs in G-C's per-platform re-measurement and is listed so it is not
    discovered there.

21. ~~Run the keyboard audit's surfaces with the pointer unplugged.~~ **Complete 2026-08-15 —
    22 of 22 rows, S-08 met, and with it `MASTER.md` §10's twelfth box.** Opened 2026-08-08 and
    closed over three sittings: nineteen rows on the 14th, the last three on the 15th once row 12
    had been re-worded (D-73) and the binary rebuilt so rows 20 and 21 had a surface to walk. Ten
    findings, all confirmed fixed by a walk rather than by a commit, **four of them found by the
    walk while three automated audits passed**. The note below is from the day it opened and its
    history is still the useful part.** ~~3 passed
    of 22, and it has already paid for itself twice.~~ The denominator
    moved on 2026-08-08 (D-70 added rows 20 and 21), and the numerator is corrected down from the
    4 written here: `docs/keyboard-audit.md` ticks rows 7, 8 and 9, and its checkboxes are the
    record. `npm run a11y`
    answers the seven **global** rules on every build and cannot answer one of the per-surface
    rows — `element.focus()` is not Tab, and nothing automated can say the order it produced is
    the order a person was reading in.

    **The total is 20 boxes, not the 19 written everywhere before today**: the rows are numbered
    1–19 and row 8a sits alongside row 8, so every "0 of 19" was counting the highest row number
    instead of the rows. Corrected in `docs/keyboard-audit.md`, and it is the second time this
    table's count has been wrong in the direction of a total — which is the argument for the gate
    line already being re-worded to need **a list rather than a total**.

    | Rows | Result |
    |---|---|
    | 7, 8, 9 | **pass** |
    | 6 — sidebar | passed, then **re-opened by D-67**, which changed the surface under the tick |
    | 3 — onboarding step 3 | **fail**, finding 4 |
    | 11 — command palette | ⌘K opens; the command rows failed, finding 5 → **D-66**, fixed and **confirmed in the app by the author**. Its own four remaining clauses unwalked |
    | 8a, and 1, 2, 4, 5, 10, 12–19 | not walked. **Row 8a is the one to watch**: a separate box from row 8, visible only on an item carrying a one-time code, so walking the detail pane of an item without 2FA skips it silently |

    **Two of the four defects so far were found by measuring rather than by walking**, with a
    throwaway script that enumerates what the browser treats as a tab stop, in document order
    (`docs/keyboard-audit.md`, findings 6 and 7). It cannot press Tab and so cannot see a focus
    trap, and it runs against the harness — but it answers "is this control in the tab order at
    all", which neither existing audit can ask. ~~Worth considering as a third audit~~ **It is
    one, since 2026-08-14 — D-71, `scripts/audits/taborder.js`, and it returned finding 10 on
    its first full sweep**: the profile popover was four tab stops where `role="menu"` promises
    one, on a surface built four days earlier and never walked. Row 20 is unchanged and still
    unticked — the fix has not been walked either. Finding 6 is the argument: `Segmented` dropped out of the tab order entirely whenever
    its `value` matched no option, and the screenshot/a11y fixture had been missing `ui_scale`
    since D-57 — so Interface size had been keyboard-unreachable in **every** harness run, and
    the Settings screen had been measured against a settings object the host cannot produce.

    **Neither defect was a keyboard defect in the way the audit assumed.** Finding 4 is two
    causes on one screen — step 3 has no text field, and `onenter` lives on the input
    (`TextField.svelte:88`), so Enter reaches nothing; and the step carries no `autofocus`, so the
    transition leaves focus on `document.body`. That second half is **finding 1 in a second
    place**: `Dialog.svelte` was fixed for exactly it on 2026-08-07 and the fix was made to the
    dialog rather than to the pattern. Finding 5 is not about the keyboard at all — the pointer
    path was identically broken — and is fixed as **D-66**.

    Rows still written as predictions, and still unearned: row 18's "Tab from Replace lands in
    the input it just opened", row 12's tag field swallowing Enter, and row 19's return from the
    native picker. **Row 12 needs a wording decision before it can be walked**: its clause "the
    generator opens from inside it and returns focus" describes a dialog that does not exist —
    `NewItemDialog.svelte:124` calls `generatePassword()` and fills the field in place. The
    author's call, like D-64, not a silent edit.
22. ~~`npm run a11y` is not in CI, and neither is `npm run shots`.~~ **Both are, since 2026-08-14
    — D-72.** The job is `a11y` in `ci.yml`: Firefox from `browser-actions/setup-firefox`, the
    three audits over twenty scenarios in both themes, then the twenty-five screens rendered and
    uploaded as an artifact. ~~It has not yet run on a runner~~ — **it ran 2026-08-16, run
    31937787913, and it failed. Four minutes seventeen, not eight, and 1 556 findings, every one
    of them `no-indicator` — 100 % of the focusable elements on every surface in both themes.**

    **Found and fixed the same day — D-92 — and the cause was neither the audit's logic nor the
    app.** The suspicion below was right about the class and wrong about the mechanism. It is not
    `focus({ focusVisible: true })`: on a fresh profile Mozilla's own build shows the
    data-collection privacy notice at startup, whatever presents it takes **window activation**
    from the content, and from there `document.hasFocus()` is `false`, `:focus` matches nothing,
    and every ring in the application is invisible to `getComputedStyle`. `contrast` and
    `taborder` do not care who is focused, which is why they were clean beside it.

    **The variable was the build, not the machine**, and that is the part worth remembering. Both
    sides are 153.0.4; this desktop runs Ubuntu's **snap**, which suppresses that notice, and the
    runner installs Mozilla's **tarball**, which does not. Reproduced here by downloading the
    runner's own build and running the audit against it — `hasFocus: false` and every probe
    dark — then bisected to a single pref, with which the full 144-surface sweep is clean on that
    build. Two days of "the audit is clean" had been measured against the one Firefox that hides
    the problem.

    Both cautions below were kept. Nothing was widened: `browser.display.show_focus_rings` would
    have gone green by forcing a ring onto anything focused, and it is named in D-92 as rejected.
    And `focus.js` now **measures its own instrument first** — a control it creates, styles by id
    and throws away, probed separately for `:focus` and for `:focus-visible` — so this failure can
    never again arrive as 1 556 findings against the application. It arrives as one line saying
    the browser drew nothing, and it still exits non-zero.

    **`MASTER.md` §10's twelve boxes are still ticked against one machine agreeing with itself.**
    The fix makes a second machine able to check them; nothing has yet re-ticked them from a
    runner, and the run that does is the one to watch. The note as written:

    Both need a headless
    Firefox and about eight minutes for a full sweep, and both are now the only thing standing
    between a token edit and a regression nobody sees — a contrast failure is invisible in a
    diff and invisible in review. Same decision as action 12 and it should be made once for
    both: a baseline and a comparison make the screenshots a test, and the a11y run **already
    exits non-zero**, so it is a CI job the day someone decides the eight minutes are affordable.
23. **The two audits measure text and focus, not everything §10 asks.** S-09's "≥ 3:1 UI
    boundaries" is not measured: the tool reads text nodes, and a hairline border's contrast
    against the surface it divides is a different walk. Named here rather than rounded up,
    because the checklist line is ticked for the half that was measured and says which half.
24. ~~Two of Phase 4's five gate lines are waiting on the author.~~ **Both answered 2026-08-15,
    D-77** — taken as proposed, split rather than relaxed, and the gate is **seven lines**. Acting
    on them cost a finding each: a third decoy count (156) plus the CRLF parsing hazard now in
    §6.9, and the discovery that the reference vault has **no reuse and one repeated zxcvbn score**,
    which added two tasks. **One thing is deliberately still blank**: S-07a has no target until the
    first `cargo bench` produces one, and the rule is that the number is written into
    `trustvault-requirements.md` the same day it is first read, with the cheap-case caveat beside
    it. A blank that stays blank past the first measurement is how a criterion quietly stops
    existing.
25. ~~The scoring/reuse fixture is the next core task, and it is not `benchfixture.rs`.~~ **Done
    2026-08-15**, `crates/trustvault-core/src/auditfixture.rs`, behind the same feature flag as the
    reference vault because it is the same risk — a fixture generator compiled into the application
    is a way to put fake items into a real vault. Twenty-one items, groups of 3/2/5/2, a password at
    every score, and every claimed score re-read from zxcvbn on each test run rather than asserted
    once. **The `cargo bench` beside it filled S-07a's blank the same day** — 61–63 ms against a
    500 ms budget, D-80. Two things the task list had not anticipated, and both were cheap only
    because they were found now: the **gate line's claim that CI re-runs the bench was false** —
    nothing in `.github/workflows/` ran `cargo bench` at all, for any of the three benchmarks — and
    the reference vault is **not finding-free**, returning ten weak rows whose passwords are eight
    characters long. What is still owed from this task is the *frontend* half: the three CI audits
    and the view draw from TypeScript fixtures, and those still know nothing about findings — that
    is the entry-check task about harness scenarios, not this one.
26. ~~`watchtower_scan` is not registered, and `// planned` in §6.9 is still accurate.~~ **Done
    2026-08-16.** The command, both `vault_status` timestamps, the status-cache write and the save
    all landed together, and the marker was deleted in the same commit — the tenth time that habit
    has held. Three things it cost that this note had not anticipated, and each is cheap only
    because it was found now:

    - **No checkbox in `phases/phase-4-watchtower.md` owned the work.** Every task there is worded
      for the core or for a surface, and the boundary between them — command, cache write, save —
      was specified in §6.9 and in no task. It is a task now, ticked, with the reason written into
      it: Phase 3's accounting lesson in the other direction, where host work sat under
      surface-worded boxes and went unticked. Here it would have been done and invisible.
    - **What to write onto an item with no password field is a decision, not an implementation
      detail** — **D-81**. `strong` on a secure note is a green pip claiming a check nobody ran.
    - **The elision proof needed a reuse group planted in the session**, or it would have been
      searching a transcript with no grouping key in it. Both harnesses now also look for any run
      of **64 hex characters**, because `carrying(SECRET)` cannot see a hash of the secret and that
      is exactly the shape §6.9 forbids. Verified non-vacuous by leaking the key on purpose.
27. ~~The Watchtower view still draws from the status cache, not from findings.~~ **Done
    2026-08-16, later the same day** — groups and rows from the report, crack times in zxcvbn's own
    words, **D-82** for the two prototype sentences that could not survive (a password length on
    screen, and an HIBP claim about a request never made). Six boxes closed. Three things it cost
    that this note had not anticipated:

    - **Where the four tiles get their numbers is a decision**, not a detail. Counting findings put
      *0 breached, 0 reused, 0 weak* above the error message on a refused scan. They count cached
      statuses now, which also keeps them in agreement with the list's pips and the sidebar badge.
    - **The error state needed a control the prototype does not draw.** It draws no failure at all,
      so *Try again* is this project's call (D-48's shape). Without it a refused scan is a dead end
      until the vault is locked and reopened — and the effect that scans marks a generation
      **attempted** rather than **succeeded**, because the alternative is a retry loop against the
      host on a screen the user is only reading.
    - **The reassurance copy had never been rendered by anything.** The only unlocked fixture has
      four flagged items in it, so `watchtowerClean` is a new scenario rather than a new sentence,
      and `watchtowerError` beside it. Sweep: **132 surface-audits, no findings**.

    **One observation left deliberately unacted on.** `ITEMS` gives `card` the status `expired`, and
    **nothing in the product can produce that** — `worst()` maps weak, reused and breached only, and
    §6.9 says the Expired group stays empty. It is the pip specimen's only shot of that variant, so
    it stays until a fixture pass decides it deliberately.

28. ~~The HIBP client needs an HTTP dependency, and there is none in `src-tauri` today.~~
    **Done 2026-08-16 — D-83, ureq 3.4 — and the client landed the same day.** The survey's
    first finding was that the premise every one of these starts from is false: `reqwest 0.13.4`
    is in `Cargo.lock`, which reads as free, and `tauri` declares it only for **Android and
    non-macOS Apple targets** — the three desktop triples have no HTTP client in the baseline at
    all. Measured per real triple, ureq adds **+12/+12/+12** crates against reqwest's
    **+30/+31/+35**, `cargo audit` is clean on every candidate lock, and `gzip` is free in crate
    count while halving the transfer. Four of the four constraints this note set were met and the
    fifth thing it did not anticipate is the one that cost the most: **the crate was the easy
    half**. ureq reads `ALL_PROXY` from the environment, defaults `https_only` to false, and
    follows ten redirects — and the first tests written for two of those three **passed with the
    override deleted** (D-84). What is still owed from this task is nothing; what it unblocks is
    `watchtower_breach_check`, which is next action 30. The original note:

    That is a prior-art survey before adoption, not a `cargo add` — maintenance, licence, platform
    support, resource cost, exit path, with the alternatives rejected in the decision log. It is
    the next repository task and it gates the whole breach-checking group. Four things this
    project's own constraints put on the shortlist before any candidate is read: it must not
    reach `trustvault-core` (N-02 — the client lives in `src-tauri/src/hibp.rs` for exactly this
    reason), it must support **connection reuse** across a thousand requests (§6.9's measurement
    is what says so), it must let the caller **set a request header** and bound concurrency, and
    its TLS story has to be the same on all three platforms or S-06's 40 MB gets an argument.
    `tauri-plugin-http` is on the list and starts at a disadvantage that D-59's survey named: a
    plugin's cost is not only its tree, and the last one rewrote `window.alert` in our page.

    **As of 2026-08-16 this is the only thing left in the phase that is code.** Seven of the eight
    unticked tasks are the breach group and the two verification tasks behind it, all of which need
    the client; the eighth is row 17's walk, which needs a person and a build. So the phase does not
    move again until this survey is done and its decision logged — and it is the author's to take,
    not a `cargo add` taken on the way to something else.

29. **`create_vault` refuses an existing path, and nothing else in the product does.** D-62 put
    the check where a vault gets written. If a later phase adds a second path that writes a
    vault file — an export, a backup, a "save a copy" — it will need its own, because the
    refusal is in `create_vault_inner` rather than in `Vault::save_to`, and it is there
    deliberately: `save_to` is what every ordinary save calls, and a `save_to` that refused an
    existing file could not save anything twice.
30. ~~`watchtower_breach_check` is not registered, and `// planned` in §6.9 is still accurate.~~
    **Done 2026-08-16.** All four things this note listed were decisions, as predicted, and all
    four are in the code with a test each: the setting is read before a client exists (and the
    test for it watches a listener rather than reading a boolean back), the vault is dropped
    across the network phase and re-taken under a **generation** check, the progress event is two
    integers on a four-worker pool (**D-85**), and a failed value is `unchecked` with its status
    untouched. Two things it cost that this note had not anticipated:

    - **The idle clock is a fifth bound and it has no surface.** The second vault acquisition
      cannot go through `with_vault`, because that touches `last_activity` — a minutes-long
      background task that resets the auto-lock timer keeps a vault unlocked for as long as it
      runs. It is a hand-written `state.with` in `commands/watchtower.rs` with the reason on it.
    - **A partial pass must not stamp the vault — D-86**, and the reason is that `unchecked` dies
      with the window while the timestamp does not. It also made the screen and the vault say
      different things on purpose, which is what **D-87**'s lede fix then had to follow.

    The original note:

    - **The setting owns the egress** (§6.9). `Settings.breach_check_enabled` is read in the
      command, not in `hibp.rs`, because a client that refuses politely is a client somebody can
      call anyway. Called with the setting off it returns `requested: 0` and every password in
      `unchecked` with reason `"off"` — and it **does not error**, because a refusal read by the
      UI as a failure is a refusal the UI will offer to retry.
    - **The vault may lock while it runs** (§6.9). `watchtower_scan` is exempt by construction —
      61 ms, holding the vault throughout — and this one is minutes. It finishes, finds the vault
      closed, and drops what it computed; it must not touch the lock timer, because a background
      task that resets the auto-lock clock keeps a vault unlocked indefinitely.
    - **Progress and bounded concurrency.** `watchtower-progress` is specified in §8 and emitted
      by nothing. The bound is a worker pool rather than a semaphore because D-83 chose a blocking
      client, and §6.9 measured 8 concurrent at 2.5 req/s against 1.87 serial — the gain is small
      and the politeness is the point.
    - **The offline path is R-25's own wording**: a failed check reports "not checked", never
      "safe". `RangeError::reason()` already answers in §6.9's vocabulary; what is undecided is
      what the *screen* says when half a check landed, and the tiles counting cached statuses
      (D-82's first bullet) is the shape that answer has to fit.

31. **The packet capture is the gate's evidence and nothing in CI can take it.** ~~It is now the
    only thing standing between this phase and its gate that is not a keyboard walk~~ — **the
    harness for it landed 2026-08-16 and what is left is the run.** `scripts/capture.sh` has four
    subcommands and only two of them need a person:

    | Command | Needs | Answers |
    |---|---|---|
    | `selftest` | nothing — it is a CI step | that the report script can fail: it breaks its own capture eight ways |
    | `probe` | network | gate line 3, which is a claim about the service and cannot be read off a capture at all |
    | `off --minutes 10` | `sudo`, a build, a person using the app | gate line 4 — S-10, R-26 |
    | `on` | the same, plus the setting turned on | gate lines 1, 2 and the countable half of 6 |

    **All four subcommands have now been run, and the two that need a person must be run again.**
    On 2026-08-16 the author took two `off` runs and one `on` run against a rebuilt binary, and
    all three are void for one reason: the sampler died on its first pass (`set -e` inherited by
    the subshell, `ss | grep` exiting 1 on the quiet case), so attribution was empty in every one.
    Fixed, with a test of its own that reproduces the failure when the fix is removed. What the
    runs are worth keeping for: the sweep over the `on` capture is **clean** — 100 needles, 1 647
    packets, no match — and that half of gate line 2 never depended on attribution; the capture
    holds five ClientHellos, two of them to `api.pwnedpasswords.com`; and D-91 came out of the
    twenty false hits. **Both runs need repeating on the fixed harness**, and the `on` one
    especially: the same wire carried Telegram, Gemini and Brave's updater, so without attribution
    "the app reached only HIBP" cannot be told from "this machine reached HIBP".

    **`probe` was run the day it was written and gate line 3's service half is measured**: 2 151
    padded rows against 1 978 unpadded for `5BAA6`, **173** of them zero-count, the padded body
    6 747 bytes larger, `vary: add-padding` and `cache-control: no-store` present, and the count
    for the SHA-1 of `password` at **52 372 427** — the number `hibp-range-5BAA6.txt` carries,
    from the live service. The line still does not tick: it asks for the padding of *the prefix in
    that capture*, and there is no capture yet. Note also that the real-row count moved again,
    1 924 → 1 978 in one day, and the decoys 163 → 173 — which is the third and fourth reading
    behind D-77's refusal to write a band into the gate.

    Three decisions came out of building it, and each is a thing the run would otherwise have
    discovered while it was too late to change: **D-88** (attribution is the process *tree*,
    because WebKitGTK does its networking in a process the main PID does not cover), **D-89**
    (TLS means the capture proves the negative and `hibp.rs` proves the positive), and **D-90**
    (the run is against the audit fixture — twelve distinct values, a genuinely pwned password,
    and every string in it published so the forbidden-string list can be generated rather than
    typed).

    **What running it takes, in order.** Build the app — `npm run tauri build -- --no-bundle`,
    the default one and not `measure`, for the reason in action 18. Then `scripts/capture.sh off
    --minutes 10`, using the app normally throughout; then `scripts/capture.sh on`, unlocking
    `/tmp/trustvault-audit.tvault` (password `correct horse battery staple`, built by the script
    if it is not there) and running the breach check to completion. Each writes a directory under
    `captures/` — **git-ignored, and that is deliberate**: an unfiltered ten-minute capture holds
    every packet this machine sent, which is not TrustVault's to publish. What leaves it is the
    verdict table, pasted into the session log below. The note as written:

    Four of the seven
    gate lines are a `tcpdump` run on the app's PID, and the same sentence that has been true since
    action 14 is true here: nothing in CI runs the app. What is different now is that the *claims*
    the capture checks are asserted offline first — `hibp.rs` asserts what was handed to the client
    and `hibp_egress.rs` asserts where it was allowed to go — so the capture is confirming a wire
    rather than discovering a design. Take it with the same build the keyboard walk uses, not the
    `measure` one.
32. **A breach check cannot be stopped once it starts.** §6.9 left the cancel out deliberately —
    "a cancel is a decision about what a half-finished check leaves behind, and it follows the
    S-07 answer rather than preceding it" — and the S-07 answer exists now (D-77, D-86). What a
    half-finished check leaves behind is settled: the hits it found, and no timestamp. So the
    question left is narrower than it was, and it is a **surface** question: closing the window or
    locking the vault already ends the check's usefulness (the results are dropped), but nothing on
    screen says so and there is no button. It is a rough edge on a vault of a few hundred
    items and a real gap on the reference vault's thousand, where §6.9's own arithmetic puts the
    check at **~400 seconds**. Worth deciding before a release, not before the gate.
33. ~~Nothing has yet run a breach check against the real service from inside the app.~~ **It ran
    2026-08-16, and it was right.** The author's `on` sitting put the audit fixture in front of the
    live service and the three items carrying `password` came back **breached at 52 372 427** — the
    published count, the same number `probe` read out of the service by curl an hour later, and the
    same number `hibp-range-5BAA6.txt` carries offline. That is **gate line 1's live half**, and it
    ticks. It is an **observation, not a measurement**: nothing in CI reproduces it, for the reason
    that has held since next action 14 — nothing in CI runs the app. What makes the observation
    load-bearing rather than anecdotal is that the count cannot come from anywhere else: the
    trimmed fixture is compiled into the *test* binary and not into the application, so a release
    build printing 52 372 427 has had a range response from `api.pwnedpasswords.com` — which the
    capture independently shows it asking for, by DNS and by SNI.

    **The parser is still the thing that was proven, and only for one response.** The original
    note's warning stands for everything it named, because none of it is visible from the outside
    of a TLS session and none of it was instrumented: whether **gzip** was actually negotiated,
    whether the `__cf_bm` cookie came back and was correctly not sent on (asserted offline, still
    never observed live), and what a **429** does to a four-worker pool. Twelve prefixes on a warm
    Cloudflare edge is the friendliest possible sample. The note as written:

    Every
    assertion behind the command is against a listener this project starts, serving a response this
    project trimmed — the same shape as next action 19's warning about the importer, and the same
    answer: the fixture proves the parser against our reading of the format, not against what
    Cloudflare serves on the day. The capture run (action 31) is where that first happens, and the
    three things to watch while it does are the ones a fixture cannot vary: a response with
    **gzip** actually negotiated, the `__cf_bm` cookie coming back and not being sent on (asserted
    offline, never observed live), and what a **429** looks like from a client making 4 requests at
    a time rather than 48 in a loop.

## Session log

### 2026-08-16, last (the audit was measured against the one browser that hides the problem)

**The a11y job's 1 556 findings were a privacy notice.** Yesterday's first runner execution
reported every focusable element on every surface in both themes as `no-indicator`, and the
reading recorded then — "almost certainly the audit rather than the app" — was right about the
class and wrong about the mechanism. It is not `focus({ focusVisible: true })`. On a fresh profile
Mozilla's build shows the data-collection privacy notice at startup; whatever presents it takes
**window activation** away from the content; `document.hasFocus()` is then `false`, `:focus`
matches nothing, and every ring in the application is invisible to `getComputedStyle`. `contrast`
and `taborder` never ask who is focused, which is exactly why they were clean beside it.

**The variable was the build, and the local machine could not have found it by looking harder.**
Both sides are Firefox **153.0.4**. This desktop runs Ubuntu's **snap**, which suppresses that
notice; `browser-actions/setup-firefox` installs Mozilla's **tarball**, which does not. So the
environment was ruled out first and it was ruled out correctly — the audit stays clean here with
`DISPLAY`, Wayland and D-Bus stripped, and clean again under `env -i` — and then the runner's own
build was downloaded and run against the same commit, where it failed on the first scenario. That
is the reproduction the fix rests on:

| Firefox 153.0.4, headless, same flags, same commit | `document.hasFocus()` | `:focus` paints | `:focus-visible` paints |
|---|---|---|---|
| Ubuntu snap — this desktop | `true` | yes | yes |
| Mozilla tarball — what the runner installs | **`false`** | no | no |
| Mozilla tarball + `datareporting.policy.dataSubmissionPolicyBypassNotification` | `true` | yes | yes |

**One pref, bisected from three, and one that does nothing.** `focusmanager.testmode` — Gecko's
own harness pref for making focus work in a window the platform never activated — was the first
guess, because the symptom is exactly what it is for. It changes nothing here, measured, and it is
not carried: a pref that fixes nothing is a charm, and the next person would have had to re-derive
that it was one.

**The other half is that the audit now measures its own instrument** (D-92). `focus.js` creates a
button, styles it by id, focuses it and throws it away, before it looks at anything in `dist/` —
two probes, one for `:focus` and one for `:focus-visible`, because those two failures need
different fixes and are indistinguishable in a findings list. When a probe fails, the run prints
one line naming the browser and saying what the findings below would have been worth, and still
exits non-zero. Verified non-vacuous the way everything else here is: the probe's own rules were
made unmatchable on purpose, and the run failed with that message instead of reporting the app.

**What is measured, on the build that failed:** the full sweep against Mozilla's tarball —
**144 surface-audits, no findings, exit 0** — and the same sweep on the snap, also 144 and clean,
so the probe costs nothing where the problem never was. Nothing in the application changed today;
the diff is `scripts/a11y.mjs` and `scripts/audits/focus.js`.

**What this did not close on 2026-08-16.** `MASTER.md` §10's twelve boxes were ticked against this
desktop and they still were — the runner was now *able* to check them, and had not yet. The gate
was untouched at **2 of 7** and Phase 4 was still **23 of 24**: every remaining line was the two
capture runs and row 17's walk. Those missing records were supplied on 2026-09-09 by user report.

### 2026-08-16 (the first real run, and three defects in the thing built to judge)

**The harness ran against the real application for the first time, and it found three defects —
all three in itself.** Nothing in TrustVault failed. Two of the three made a capture report say
less than the capture actually held, which is the direction that matters: a verification tool that
under-reports is a tool that certifies silence.

**The sampler stopped after one pass, in every run.** `set -euo pipefail` is inherited by the
subshell, and `ss -tunapH | grep -E "$pattern"` exits 1 when the process tree holds no matching
socket — which is **exactly the expected state of an `off` run**. So the loop died 200 ms in, all
three captures had one heartbeat and no socket rows, and attribution was empty in every one of
them. The bug is only reachable in the quiet case: the busy case matches, grep succeeds, the loop
lives. What caught it is the **vacuity guard written hours earlier** — without it, the `off` run
would have reported *zero packets attributed to the app: pass*, which is the phase's headline gate
line passing on a capture that watched nothing. The sampler is a function now with a test of its
own: a process holding no socket must produce more than one heartbeat, and a process holding one
must have it recorded. Verified non-vacuous by removing the override again — 1 heartbeat, the
exact failure.

**The lesson is D-47's shape, one phase on.** The report script had a self-test from the day it
was written; the shell script had none. So the tested half was correct and the untested half
stopped after 200 ms in every run, and the harness's own test suite could not see it by
construction — the same sentence that was written about `ipc_session.rs` and the `_inner` split.

**The sweep reported twenty hits and every one of them was the hostname.** `password` is a
substring of `pwnedpasswords`: all fifty-four occurrences in the capture were inside
`api.pwnedpasswords.com`, in the DNS question, the DNS answer and the SNI. **D-91** masks the
label and prints how many occurrences it excused, rather than dropping the needle that matters
most.

**ClientHellos were parsed only from attributed packets**, so when attribution failed the
destination check reported *0 ClientHello(s) observed* on a capture holding fifty DNS messages for
R-25's host. Parsed over every packet now, with attribution deciding a "the app's?" column and the
whole inventory printed. A check that goes quiet when its input is missing is worse than one that
fails.

**What the `on` capture does say, re-analysed with the fixes.** The forbidden-string sweep is
**clean** — 100 needles across 1 647 packets and 1.1 MB, no match once the hostname is excused —
and that half of gate line 2 holds regardless of attribution, because the sweep never depended on
it. The capture holds five ClientHellos: two to `api.pwnedpasswords.com` (both resolved addresses)
and three to `api.telegram.org`, `gemini.google.com` and `go-updater.brave.com`, which are other
	applications on this desktop. **That last fact is why this run was not salvaged at the time**:
	without attribution, "the app contacted only HIBP" could not be told apart from "somebody on this
	machine contacted HIBP", and a machine with a browser open is not a quiet wire.

**Neither capture run was gate evidence on 2026-08-16.** The `off` run's verdict was refused by the
guard; the `on` run's destination and request-count checks failed on missing evidence. What
survived from that day was the sweep, the ClientHello inventory, D-91, and the knowledge that the
app reached the live service. The later tested evidence is recorded on 2026-09-09 by user report.

**Row 17 was not recorded on 2026-08-16.** `docs/keyboard-audit.md` was last written at 11:19,
before the sitting, and its box stayed open with no finding beside it. The walk's evidence was
recorded later, on 2026-09-09 by user report.

**The branch is pushed and CI ran on it for the first time** — `feature/phase-4-watchtower` had
never left this disk. Dispatched with `workflow_dispatch` rather than opened as a PR, because a
phase PR's body carries the gate evidence and there is none yet; the same way Phase 2's runs were
taken. Run **31937787913**: eight jobs green, one red, and both results are worth more than the
tally.

**Gate line 5 ticks — S-07a on a machine that is not this desktop.** The benchmark ran on a
GitHub runner: **62.5 ms against the 500 ms budget** for 1 000 items, beside 2.7 ms for the audit
vault. The line has needed exactly this since D-80, because a reading cannot fail a budget derived
from it. The runner and this desktop agree to within a millisecond, which says something the
budget does not: D-80's 8× headroom was sized for a machine that might be much slower, and the
first other machine to run it was not. The two vaults in one run also produce a number neither
gives alone — 0.062 ms per password on the reference vault against **0.128 ms** on the audit one,
so a password zxcvbn's dictionaries actually match costs **2.1×** one they do not. That is the
"this fixture is the cheap case" caveat turned into a multiplier, and 2.1 is a great deal smaller
than the caveat left room for. The capture report's self-test passed there too, all nine cases.

**Gate line 1 ticks as well, and it is the author's eyes rather than a file.** During the `on`
sitting the app put the audit fixture in front of the live service and reported the three items
carrying `password` as breached at **52 372 427** — the published count, the same number `probe`
read by curl an hour later, and the same one the trimmed fixture carries offline. That closes the
live half of the line whose mocked half `hibp.rs` has held since the client landed. It is an
**observation, not a measurement**, in the same class as G-B′'s fifth line: nothing in CI runs the
app. What makes it evidence rather than an impression is that the number has nowhere else to come
from — `hibp-range-5BAA6.txt` is compiled into the test binary and not into the application — and
the capture shows the request being made, by DNS and by SNI, even though it cannot say whose
socket carried it.

**The a11y job failed on its first runner execution, and it is not an application regression.**
Next action 22 said "the first PR is what proves the eight minutes and the Firefox install" — it
proved four minutes seventeen, and then failed: **1 556 findings, every one of them
`no-indicator`, which is 100 % of the focusable elements on every surface in both themes.** A
result that uniform is a statement about the environment, not about the app, and three things
support that reading: `contrast` and `taborder` were **clean on the same pages in the same run**,
the local sweep of the same commit is clean, and both sides are **Firefox 153.0.4 headless with
the same flags**. What differs is the machine, so what is suspect is `focus.js`'s central
assumption — `element.focus({ focusVisible: true })` producing a `:focus-visible` paint — holding
on this desktop and not on a runner. Unpinned, and it is a next action rather than a guess written
into a decision.

### 2026-08-16 (the capture harness, and the evidence that is not a person's grep)

**Phase 4 was 23 of 24, and the last box was a keyboard walk.** The packet-capture harness is
written: `scripts/capture.sh` (`selftest`, `probe`, `off`, `on`), `scripts/capture-report.py`
under it, and `crates/trustvault-core/examples/audit-vault.rs` in front of it. What was owed then
was the *harness*; the later run evidence is recorded on 2026-09-09 by user report.

**The reason this is a script and not a note.** Four of the seven gate lines rest on a capture,
nothing in CI runs the application, and gate line 2 is four **negative** claims about bytes — no
password, no full hash, no title, no vault or item identifier. A negative claim checked by a
person typing strings into a grep quietly narrows to whatever they remembered to type. So the
forbidden-string list is **generated from the fixture that produced the vault**: every password,
title, username and item id, plus each full SHA-1 in upper hex, lower hex and **raw binary** —
the shape a hex-only grep misses — and the report searches all of them across the whole capture.

**Three decisions, and each one is a thing the run would have discovered too late.**

- **D-88** — attribution follows the **process tree**, not the PID. A Tauri app on Linux is
  WebKitGTK and WebKitGTK does its networking in a separate `WebKitNetworkProcess`: a capture
  attributed to the main PID would have watched the process least likely to open a socket and
  missed the one most likely to. The capture itself is taken with **no BPF filter at all**,
  because a negative claim is worth nothing if something was excluded before it was made. What it
  costs is stated rather than hidden — 200 ms socket sampling can miss a short-lived socket, so
  the sweep and the hostname search run over every byte, not over the attributed subset.
- **D-89** — the capture proves the negative and `hibp.rs` proves the positive. The request is
  inside TLS and no capture reads it. Both fixes are worse than the gap: a key-logging build means
  patching ureq and shipping a password manager that can dump its own session keys, and a
  TLS-terminating proxy measures a client configured differently from the one that ships. The
  report says which half is which, in its own last section.
- **D-90** — the run is against the **audit fixture**, not the reference vault and not the
  author's own. Twelve distinct values across twenty-one password fields makes S-07b's "one
  request per distinct value" countable *and* distinguishable from a per-field bug; `password` is
  in it, so gate line 1's live half lands in the same run; and every string in it is published, so
  the manifest can be generated. The author's own vault is the one the capture cannot use, for the
  reason that makes it realistic.

**The report script breaks its own capture eight ways, and it is a CI step.** `--self-test` builds
a pcap by hand — DNS on loopback, a ClientHello with SNI, a request-sized record and a large
response — then plants a password, a SHA-1 in hex, the same SHA-1 as raw bytes, a ClientHello
naming another host, and app traffic during an `off` run, and demands the matching check fail each
time. It needs no root, no network and no display, so unlike the capture it runs on every push.
Two of the eight found something while being written: the "the same traffic under `off` fails
everything" assertion was **wrong rather than failing** — the sweep passes on that traffic, and
correctly, because there is no password in it — and an over-broad expectation is the kind that
gets loosened later until it means nothing.

**The pcap reader is stdlib, and it was cross-checked against the tool it is imitating.** No
tshark, no dumpcap and no dpkt on this machine, and a piece of gate evidence that needs a package
installed on the day is a piece of gate evidence that gets skipped. The reader parses classic pcap
and LINUX_SLL2 by hand; the check that this is not fiction is that **`tcpdump -r` reads the
synthetic captures the self-test writes** and decodes the DNS question, the addresses and the TCP
flows exactly as intended. Reading a file needs no root, so that cross-check is repeatable here.

**`probe` was run, and gate line 3's service half is measured.** Against `5BAA6`: **2 151 padded
rows against 1 978 unpadded, 173 of them zero-count**, the padded body **6 747 bytes larger**,
`vary: add-padding` and `cache-control: no-store` present, and the count for the SHA-1 of
`password` at **52 372 427** — the number in `hibp-range-5BAA6.txt`, from the live service, on a
run this repository can repeat. The line **does not tick**: it asks for the padding of *the prefix
in that capture*, and there is no capture yet. One thing worth recording beside it: the real-row
count moved **1 924 → 1 978** in a single day and the decoys **163 → 173**, which is the third and
fourth reading behind D-77's refusal to write a row-count band into a gate line.

**`captures/` is git-ignored, and it is the second-strongest reason in that file after the vault
lines.** An unfiltered ten-minute capture holds every packet this machine sent — other
applications' traffic, the names it looked up, its addresses. None of that is TrustVault's to
publish. What leaves the directory is the verdict table, pasted here by hand, which is what the
gate asks for anyway.

### 2026-08-16 (the breach check crosses IPC, and the sentence a screenshot caught)

**Phase 4 is 22 of 24, and what is left in it is not code.** `watchtower_breach_check` is
registered, the `// planned` marker deleted in the same commit — the eleventh time that habit has
held — and with it the last two code boxes in the phase: the opt-in setting and the offline paths.

**The command is four bounds the client deliberately does not own**, and each one cost a decision
or a test rather than a line. The **setting** is read before a client is constructed, so with
breach checking off there is no object in the process that could have sent anything; the test for
that is D-84's shape rather than a boolean read back — a listener at the only address the client
could reach, asserted never contacted, and verified by deleting the check, which reaches it. The
**concurrency** is a pool of four (**D-85**), a bound and not a target: §6.9's own numbers are
eight concurrent for 1.34× the serial rate, so the rate belongs to the service and the only real
question is how large a burst to impose on it. The **vault** is held to read the queries, released
across the whole network phase, and re-taken to write — and refused if the generation moved,
because a vault that locked and was opened again mid-check is a different vault, and writing into
it would file one vault's verdicts in another's status cache. The **idle clock** is the one with no
surface at all: the second acquisition does not go through `with_vault`, because that touches
`last_activity`, and a minutes-long background task that resets the auto-lock timer keeps a vault
unlocked for as long as it runs.

**D-86 is the decision that was not in the task list and is the most load-bearing.**
`last_breach_check_at` is stamped only by a check that **finished**. The timestamp is the only part
of a breach check that survives the window — `unchecked` lives in the response — so a pass that
reached three values out of a thousand and stamped *today* would have tomorrow's reader told this
vault was checked. Hits from a partial pass are still written, because a breach found is a breach
found. Verified non-vacuous the usual way: stamping unconditionally fails two tests by name, one in
the core and one in the command.

**Three tests that would each have passed for the wrong reason, and did not.** The locked-vault
refusal now has two shapes rather than one — nothing open, and *something else* open — because the
first cannot see the generation check at all: deleting that check leaves the "is anything open"
test green and only the reopened-vault test red. `ipc_session.rs` drives a **real range exchange**
against a listener it starts, because §6.9's elision bullets are about a check that made requests,
and it then reads the transcript for hex runs of **20** characters rather than the scan's 64: a
SHA-1 is 40 characters and a prefix is 5, so the run length that catches a leaked reuse key catches
none of them. The 20 was itself checked by lowering it to 8, which fails on the UUIDs — a filter
matching nothing would have been a test asserting over an empty loop.

**The screenshot caught a false sentence, which is the third time that harness has paid for
itself.** The lede's closing clause said *nothing on this screen has left this device* whenever the
vault carried no breach-check timestamp — true yesterday, false as of today for exactly one state:
a **partial** check writes no timestamp (D-86) and has still sent prefixes for everything it
reached. The `watchtowerPartial` shot put that sentence directly above a row reporting what came
back. It reads off a request having been made now, and **D-87** carries it together with the other
copy departure this screen needed: the breached row says *"Found in 1,246 breach records"* where
the prototype names an incident and a year the range API cannot give us.

**Two harness scenarios rather than one**, and both are driven through the button. A scenario that
set the report directly would photograph a screen no sequence of clicks can produce, and the button
is the only thing in the product that starts a check — R-26 makes this the one command a person has
to ask for. The first version of the drive matched on *"Check now"* and clicked **nothing** in the
scenario that carries an earlier timestamp, because that button reads *Check again* — it
photographed the state before the run, which is how the shot came to be worth taking. Sweep:
**twenty-four scenarios, 144 surface-audits, no findings**, in both themes.

### 2026-08-16, the client (the survey whose premise was wrong, and two tests that proved nothing)

**Next action 28, done as written: a survey before a `cargo add`.** The result is **D-83** — ureq
3.4, `default-features = false, features = ["rustls", "gzip"]`, bundled webpki-roots — and the
numbers behind it are in the decision log rather than here. What belongs here is the finding that
came first and nearly went unnoticed.

**The premise was false, and it was measured rather than assumed.** `Cargo.lock` carries
`reqwest 0.13.4`, `hyper`, `tower` and `tokio`, which reads as reqwest already being paid for. It
is not: `tauri` declares reqwest under `cfg(any(target_os = "android", all(target_vendor = "apple",
not(target_os = "macos"))))`. On the three desktop triples the baseline has **no HTTP client at
all**, and `cargo tree --target all` — the obvious command — reports the opposite. Measured per real
triple by inserting each candidate and diffing the tree: **ureq +12/+12/+12**, **reqwest+rustls
+30/+31/+35**, **attohttpc +12** (MPL-2.0, aws-lc-sys, and no connection pool), **tauri-plugin-http
+26 including reqwest 0.12 beside tauri's 0.13**. `cargo audit` clean on every candidate lock. The
first draft of this session's recommendation was *reqwest, because it is free*; it was free in the
wrong column.

**Three measurements of the live service, taken because the survey needed them.** A **fourth**
decoy count for `21BD1` — 163, after 110, 125 and 156 — which is more evidence D-77 was right to
delete the row-count band rather than widen it. **No `User-Agent` is required**: the range endpoint
answers 200 with the header suppressed entirely, so the UA requirement everyone quotes belongs to
the authenticated v3 API and ours is courtesy. And **Cloudflare sets a `__cf_bm` cookie** on the
response, which is why "no cookie is sent back" is an assertion in the request test rather than a
default nobody checked.

**gzip is free and does not undo the padding.** It costs zero additional crates — `flate2` is
already under tauri — and halves the transfer. The question worth asking was whether compressing a
response whose size is deliberately padded gives the size signal back; measured across five
prefixes, the padded spread is **1.30× uncompressed and 1.31× gzipped**, so it does not. That was
the answer the measurement gave rather than the one the reasoning predicted, and the feature is on
because of it.

**Then the client, and the part that was actually hard.** `src-tauri/src/hibp.rs`, ~200 lines, and
the crate choice was the easy half. The three settings that decide what leaves the machine are all
ureq defaults pointing the wrong way: `Config::default` calls `Proxy::try_from_env`, `https_only`
is **false**, and redirects follow **ten** deep. Each is overridden, and each override has a test.

**Two of those three tests passed with the override deleted — D-84.** The proxy one asserted
`client.agent.config().proxy().is_none()`, which is what anybody writes and which measures *this
machine's shell*: `try_from_env` returns `None` when no proxy is set, so the assertion held with
`.proxy(None)` removed. The redirect one accepted `Err(Http(302)) | Err(Offline)`; with
`max_redirects(5)` the followed request landed on a dead port, came back `Offline`, and satisfied
the disjunction. Both are rewritten to start a listener at the place the request must not reach and
assert it is never contacted, and both now **fail when their override is removed** — checked, not
assumed. The proxy one needs `ALL_PROXY` set in the process, so it lives in
`tests/hibp_egress.rs` as its own binary rather than racing every other test that builds a client.
This is D-47's lesson in the phase whose gate is a packet capture: a capture taken on a machine
with no proxy set would have recorded a perfectly clean run either way.

**Where the SHA-1 is computed was a decision, not a detail.** It is in `trustvault-core`, which is
D-78's argument one command further on — hashing every password means reading every password, and
the core already holds them. What crosses into `src-tauri` is `BreachQuery`: a prefix that R-25
permits to leave, a suffix with **no accessor at all**, and the ids. `hibp.rs` cannot log or
serialize the suffix because it has no way to obtain one; the only thing it can do is ask
`matches()`. Same shape as `GroupKey`, one crate out. Two smaller things fell out of writing it: an
**empty password field sends no prefix**, because SHA-1 of the empty string is a fixed value and
sending it is a fact about the vault on a request that carries none; and a **zero-count row is
never a hit**, because padding rows carry random suffixes and treating one as a match would report
a breach on the strength of a decoy.

**The fixture is a trimmed real response, not a synthesized one.** `hibp-range-5BAA6.txt` — rows,
counts, CRLF endings and zero-count padding all off the live service today. It carries the
published SHA-1 of `password` at **52,372,427**, which makes the gate's first line met in its
mocked half, offline, on every push. The suite needs no network and nothing in it is `#[ignore]`d.

**Boxes: 19/23.** Four ticked, one added-and-ticked because no box owned the egress tests — the
denominator has now moved twice in this phase for the same reason, and both times it was found by
doing the work and then looking for the box. What is left is four: the opt-in setting and the
offline paths, which both need `watchtower_breach_check`; the packet capture; and row 17's walk.

### 2026-08-16, later (the view reads a real report, and two sentences the prototype cannot keep)

**Phase 4 is 14 of 22, and the Watchtower screen is no longer drawn from a cache alone.** The four
view boxes, the crack-time box and the harness-fixture box all close in one sitting, because they
were one piece of work: the surface existed from D-36 and what it lacked was data.

**The screen reads from two sources, and which number comes from which was the session's real
decision.** The groups and their rows come from the `watchtower_scan` report — the fresh answer, and
the only thing carrying crack times and who a password is shared with. The four **tiles** count items
by their cached status instead. *Safe* had to come from there in any case (§6.9 gives the report no
row saying strong, so clean exists only as an absence), but the first version had the other three
counting findings, and the case that killed it is the one worth keeping: on a **refused scan** the
tiles read *0 breached, 0 reused, 0 weak* above an error message — a clean bill of health issued by a
check that never ran. Counting the cache instead means the tiles agree with the pips and the sidebar
badge always, each item is counted once under its worst verdict, and a failed scan shows the last
scan's conclusions with a sentence saying today's did not run. The groups stay per-finding and
overlap, which is where an item that is both breached and reused appears twice.

**D-82 — two sentences from the prototype do not survive contact with this product.** The weak
row's *"10 characters · word + year"* is a **password length on screen**, and D-32 made every mask a
fixed width precisely so a length never crosses this boundary; it reads *"Crackable in 31 minutes"*
instead, which is R-24's own requirement and a better sentence besides. And the lede's "hashes
matched against Have I Been Pwned" describes a request that has never been made — breach checking is
off by default — so it says what actually happened. The seventh time this project has deleted a true-
looking sentence with nothing behind it.

**Three states have scenarios now where one had none.** Never scanned, scanned and clean, and a
scan the host refused — the last with a *Try again* button, which is the only control on this screen
the prototype does not draw, because the prototype draws no failure. Without it a refused scan is a
dead end until the vault is locked and reopened, and the effect that runs the scan deliberately marks
a generation as **attempted** rather than **succeeded**: marking on success would leave a failed scan
eligible to re-run the instant the in-flight flag cleared, which is a retry loop against the host on
a screen the user is only reading.

**The empty state was written months ago and nothing could reach it.** The only unlocked scenario in
the harness is a vault with four flagged items, so *"Watchtower has nothing to report."* had never
been rendered by anything. `watchtowerClean` is one of two new scenarios — a scanned vault, two
strong items, an empty report — and the copy is now photographed in both themes and audited like
every other screen. That is the Trash lesson applied before it bites rather than after.

**The report fixture is built to agree with the item fixture**, which took more thought than writing
it. The two halves of the screen read from different places, so a report naming items whose statuses
disagree would photograph a screen the product cannot produce. `bca` therefore carries **two**
findings — breached and reused — which exercises the report-keeps-both / cache-picks-one asymmetry,
and the reuse group has two members because *"Same as …"* with nothing after it is a sentence, not a
row.

**Row 17 of `docs/keyboard-audit.md` is un-ticked, as the entry check predicted.** Worth more than
the bookkeeping: the row asks for ↑/↓ between findings, and **nothing on that screen answered an
arrow key** — what the 2026-08-14 tick actually recorded was Tab and Enter. The rows are a real
`<ul>` now with `ItemList.svelte`'s arrow handling, every row still a tab stop, so the clause is
implemented rather than assumed. S-08 goes back to 21 of 22 until somebody walks it.

**One observation this session did not act on.** The item fixture gives `card` the status `expired`,
and **nothing in the product can produce that** — `worst()` maps only weak, reused and breached, and
§6.9 says so. It is the pip specimen's only shot of that variant, so it stays; the note is here so
the next fixture pass decides it deliberately rather than discovering it.

**Nothing new crosses IPC for any of this.** The view calls `watchtowerScan()` once per screen
opening and re-scans only after a mutation, because the scan saves the vault — navigating away and
back must not re-encrypt the file. The full sweep after all of it is **132 surface-audits, no
findings** — twenty-two scenarios, three audits, both themes, up from 120 on 2026-08-14 — and the
twelve new ones are the two Watchtower scenarios added today.

### 2026-08-16 (the scan crosses IPC, and the task nobody had written down)

**`watchtower_scan` is registered, and the phase is 8 of 22.** The core half has existed since the
15th; what landed today is the boundary — the command, the status-cache write, the save, and the two
`vault_status` timestamps — plus the elision proof the entry check asked for. Two tasks tick and the
denominator moved by one, which is the session's first finding.

**Nothing in the phase document owned the work.** Every task in `phases/phase-4-watchtower.md` is
worded either for the core (*"zxcvbn integrated in the core"*) or for a surface (*"crack-time
estimates rendered"*), and the boundary between them was specified in `docs/ipc-contract.md` §6.9 and
in **no checkbox at all**. It is Phase 3's accounting lesson arriving from the other side: there, host
work sat under surface-worded boxes and stayed unticked while being done; here it would have been done
and left no trace. Added as a ticked task with the reason inside it rather than counted quietly under
a neighbouring box.

**D-81 — an item with no password field keeps the status it had.** `Report` carries no row saying
`strong`, so absence from it is the only evidence a field is clean and the caller has to write that
word. The question the code forced was what to write onto an item there was nothing to score:
a secure note, a Wi-Fi entry with only an SSID. `strong` is a **verdict** — the list draws a green pip
and the word beside it — so writing it would put a passed check on every note in the vault, which is
D-49's *Copy & autofill* one screen further on. They stay `unknown`, the cost is named rather than
hidden (`ItemStatus` has one word for "never scanned" and none for "nothing to scan"), and
`last_scan_at` is what tells the two apart.

**The two timestamps landed with the command, not after it** — `vault-format.md` §6.7, absent when
never, no version bump, and the known-answer vectors still pass untouched because an unscanned vault
writes no key. `last_breach_check_at` is carried while nothing writes it: one absent key now against
a second format change for the same feature later.

**The elision proof needed a group planted in it.** The task read "a Watchtower finding is proven
elided in `ipc_session.rs`", and the honest version of that is not one assertion but two. The session
adds a **twin item carrying the value the edit stored**, because a report with no reuse group has no
grouping key in it and the check would have measured nothing. And the transcript's existing searches
**cannot see the thing §6.9 actually forbids**: `carrying(SECRET)` looks for plaintext, while a leaked
grouping key is a SHA-256 — 64 hex characters containing none of it. Both harnesses now search for any
hex run that long, in `ipc_audit.rs` over the response and in `ipc_session.rs` over the transcript.
Verified non-vacuous the way this project verifies things: a `group` field added to `Finding` on
purpose, and the audit fails naming two of them.

**What the scan does that no other vault-class command does: it reads every password in the vault.**
That is why it refuses while locked and why the refusal is in the locked-command list rather than
assumed from `with_vault` — a scan that ran while locked would be the whole vault decrypted by a
command whose response looks harmless in every check the harnesses make.

**The view is next and it inherits fixture work, which is why it did not happen today.**
`scripts/harness.mjs` gained the two timestamps with the DTO (finding 6's rule: a field added to the
DTO and not to the fixture quietly changes what every surface is measured against), but it answers
`watchtower_scan` with nothing. A report fixture has to agree with the items it names, and `ITEMS`
holds exactly **one** `reused` item because it was built to show each status once — a stubbed report
naming one member of a reuse group would draw a shape the product cannot produce. Half a fixture is
what finding 6 punished, so the view task owns both halves.

### 2026-08-15, last (a fixture with something to find, and the blank D-77 left)

**Both tasks the re-worded gate added are closed, and the phase is 6 of 21.** One is a vault built to
be found in; the other is the number that criterion was written without.

**`auditfixture.rs` — twenty-one items, and every count in it was worked out by hand.** Reuse groups
of 3, 2, 5 and 2; a password at every zxcvbn score; one item storing one value in two of its own
fields; one item carrying no password at all. `benchfixture.rs` could not stand in for any of it and
its own source says why — `pw-{index}-xK9` for every item is no reuse and one shape zxcvbn's
dictionaries never match. It lives behind the **same feature flag**, because it is the same risk:
a fixture generator compiled into the application is a way to put fake items into a real vault.

**The scores are read from zxcvbn on every test run rather than asserted once.** That test is the
reason the table is worth writing down: zxcvbn is a dictionary plus a set of matchers, both of which
move between releases, and a fixture still claiming a spread across five scores while holding three
would make R-24's tests pass by having nothing in them. When it moves, the test names the row.

**One row exists only to fail.** *Northwind Mail* stores `priya.raman.2024` and its username is
`priya.raman` — **score 4 with no context, 2 with the item's own words**. It is weak only because
`scan` passes the title and username to zxcvbn, which is D-12's amendment, and nothing else in the
project would notice that argument being dropped. Verified by dropping it: two tests fail, one of
them naming the row. All seven tests passed on their first run, so every one of them was broken on
purpose before being believed — Phase 2's harness rule, applied to a fixture.

**S-07a is 61–63 ms, and its budget is 500 ms — D-80.** The blank D-77 left is filled the day its
first reading was taken, which was D-77's own rule. The budget is 8× the reading and the gap is
itemised rather than left as slack: **×1.9** because the reference vault is zxcvbn's cheap case and
the same benchmark scans the audit fixture at 0.118 ms a password against 0.061 ms; **×2** for a
machine that is not this desktop; and the rest is headroom to the point where a command that returns
reads as a hang — the local scan reports **no** progress and cannot, `watchtower-progress` being the
breach half's. Half a second is that line, and it is the same order as S-03's deliberate 511 ms
unlock: the scan is never what makes opening a vault feel slow.

**The benchmark fails rather than prints, and that is the difference between a criterion and a
sentence.** `kdf.rs` and `search.rs` print, and they are right to — neither can run anywhere but this
desktop. This one runs anywhere, so it is a CI step. Which is where the session's finding is: **the
gate line said "it is a `cargo bench` line, so CI re-runs it" and nothing in `.github/workflows/` ran
`cargo bench` at all**, for any of the three benchmarks. The claim was written on 2026-08-15 and was
false on 2026-08-15. It is true now for this one; the other two stay manual and stay labelled as
manual.

**Two things the measurement found that nobody was looking for.** The budget is per **1 000 items**,
not per vault: at 0.12 ms a password on the expensive side, a 10 000-item vault crosses the line and
needs progress events the local half does not have. And a scan of the reference vault is **not
clean** — ten weak findings, the single-digit indices, whose `pw-{index}-xK9` is eight characters and
scores 2. Anything later asserting that vault scans clean would be asserting something false, so it
is in `trustvault-requirements.md` beside the number rather than in a terminal nobody kept.

**The S-07a gate line still does not tick.** A reading cannot fail a budget derived from it. What
makes it a gate line is a run that could have failed — the scan reachable through `watchtower_scan`,
and the benchmark re-run by a machine other than the one that set the number. Both are ahead, and
neither is ticked in advance.

### 2026-08-15, the scan (a threshold set by two failing tests, and one number called two words)

**R-23 and R-24's core half exists** — `crates/trustvault-core/src/watchtower.rs`, 14 tests, 96.79 %
lines. Nothing in its dependency tree can open a socket, which is the shape D-76 split the commands
for rather than a promise about a branch.

**The scoring was already written and in the wrong crate — D-78.** `score_password` has served
onboarding's meter from `src-tauri` since Phase 2, and the comment at the top of that file gives a
reason that is still correct and does not answer this question: the dictionaries are ~400 kB and the
**webview** must not parse them on every cold start. Host versus core is decided by something else —
scoring every password means reading every password, and from `src-tauri` that is a thousand
plaintext values lifted across a crate boundary to be scored and dropped, when the crate that owns
every plaintext byte is one call away. The second half is D-44's argument arriving one phase later:
the score→word mapping and the crack-time channel were about to exist twice. `score_password` is a
pass-through now, and the generator and `ipc_session.rs` call the core directly.

**The threshold was found by two tests failing, and that is the session's real finding — D-79.** The
first draft set the weak threshold at score ≤ 1, silently, on the assumption that the meter's "Weak"
band and Watchtower's "weak" group were the same line. Two tests written from that assumption failed,
and measuring instead of adjusting them gave:

| Password | Score | Cracked in |
|---|---|---|
| `password` | 0 | less than a second |
| `hunter2` | 1 | less than a second |
| `Tr0ub4dour&3` | **2** | **31 minutes** |
| `Jakarta2019!`, told the vault is "Jakarta" | **2** | **16 minutes** |
| `Jakarta2019!`, told nothing | 3 | 3 hours |
| `correct horse battery staple` | 4 | centuries |

The Watchtower view has read *"Crackable in a matter of hours"* under its Weak group since D-36 drew
it from the prototype. **That sentence describes the 31-minute row and not the under-a-second ones**,
so the threshold was already decided by copy written months earlier by somebody not thinking about
thresholds. zxcvbn agrees from the other direction — 3 is its first score meaning offline-resistant,
and a breach corpus is offline by definition. The boundary is now a test of its own, so moving the
constant has to be a decision rather than an edit nothing notices.

**And it left one number called two words on two screens.** Score 2 renders *Fair* on the strength
meter and lands in *Weak passwords* here. Both defensible alone — the meter encourages a password
being typed, Watchtower judges one already in use — and together they are the application
contradicting itself in front of the user. Raised as an open question with three shapes to choose
between, because `MASTER.md` §2 makes status wording binding and this is exactly the class of edit
that should not be made by whoever happened to notice.

**Two things the contract had not specified, decided here and written into the code's own comments.**
A field can be both weak and reused: the report carries **two findings** and `ItemStatus` ranks,
because the report is where nothing is lost and a cache is where something has to be picked. And an
item holding one password in two of its own fields is **untidy, not reused** — the alternative puts a
row on screen telling the user an item shares a password with itself.

### 2026-08-15, later still (both handed-back gate lines come back, and each one cost a finding)

**The author took both proposals as written, and neither was a paperwork edit — D-77.** That is
D-64's lesson holding for the second time in this project: a proposal is written from the same
reading that produced the line it replaces, and nothing re-checks it until somebody tries to act on
it. Acting on these two returned a finding apiece, both before a line of Watchtower code exists.

**Gate line 2, re-measured against the live service before being written down.** Prefix `21BD1`,
`Add-Padding: true`: **2 080 rows, 156 of them zero-count decoys, 81 706 bytes**; the same prefix
unpadded, **1 924 rows, 75 622 bytes**; the real suffixes byte-identical once the CRLF is stripped.
That is the **third** decoy count for one prefix in one day — 110 at the entry check, 125 hours
later, 156 now — over 1 924 real rows that have not moved once. The re-measurement was meant to
confirm the proposal and instead strengthened its own argument: a criterion naming a row count
cannot be held by a service that picks the number afresh per response. R-25 now carries all three
numbers rather than one.

**And the fetch that confirmed it found the parsing hazard.** The response is CRLF-terminated with
no terminator on its last line. Split on `\n` and every row keeps a trailing `\r`; the suffix
compare still passes because it is the half before the colon, and the **count parse fails on every
row** — which surfaces as "no breach found", not as an error. It is in §6.9 now, which is the whole
value of writing the contract first: the cheapest place to record it was a document, five days
before `hibp.rs` exists to get it wrong.

**S-07's split found the fixture nobody had questioned.** S-07a is the local scan measured against
the reference vault, so writing it meant reading `benchfixture.rs` — every one of its 1 000 items
carries `pw-{index}-xK9`. No two share a value, so **R-23 has nothing to group**, and every value is
the same short shape with nothing for zxcvbn's dictionaries to match, so **R-24 is one score
repeated a thousand times**. The vault four criteria already depend on is a fine timing floor and
**not a fixture either scoring requirement can be measured against**. Two tasks rather than one: a
separate scoring/reuse fixture, and the bench that sets S-07a's blank number carrying the caveat
that it is zxcvbn's cheap case.

**What the gate looks like now: seven lines, not five, and none of them relaxed.** Line 2 became
*what leaves* and *what returns*. S-07 became S-07a — a `cargo bench` line CI re-runs, with **no
target until the first measurement sets it**, per S-03's precedent — and S-07b, the network half
stated as behaviour rather than as a clock, because a stopwatch there measures the user's link and
HIBP's cache. The entry check is five of six; the sixth is D-74's and stays open for the life of the
phase.

### 2026-08-15, later (the contract before the code, and the second gate line it handed back)

Phase 4's first task, and the one its entry check added first: **`docs/ipc-contract.md` §6.9 is
written before any Watchtower code exists.** Two commands marked `// planned`, one event, one
setting, two `vault_status` timestamps. The marker was verified load-bearing the way this project
verifies things — deleted on purpose, harness fails naming `watchtower_scan`, restored.

**The branch is cut from Phase 3's, not from `development` — D-75.** It is D-74 one level down:
the phases overlap, so the branches do. The cost is a merge order that has to be remembered, and
it is written into the decision rather than into somebody's head.

**Two commands rather than one — D-76, and it is the session's real decision.** `watchtower_scan`
is local and touches nothing; `watchtower_breach_check` is the only socket in the product. S-10
asks for zero packets when breach checking is off, and with one command that claim is a branch
somebody must keep taking correctly — with two it is a command nobody calls. Same move as putting
the clipboard write in Rust: make the guarantee a property of the shape, not of a line.

**Writing it cost a gate line, which is the point of writing it first.** S-07 asks for a full scan
of the reference vault including HIBP in ≤ 10 s. The reference vault has **1 000 distinct**
passwords, so a full breach check is 1 000 range requests, and one-per-distinct-value is the floor
rather than the naive version. Measured against the live service the same day: **1.87 req/s** over
one reused connection, **2.5 req/s** at concurrency 8 — 2× for 8× the parallelism, so the
handshake is not the limit. **≈ 400 s and ≈ 80 MB** for the full vault; ten seconds buys about
**25** passwords. The criterion is off by 40×, and no implementation quality closes that: it would
need 100 requests per second sustained at a free service. Raised in the open questions with a
split proposal, not edited here — **the second of Phase 4's five gate lines to be handed back in
its first day**, and the fifth criterion in this project to be falsified by somebody checking it.

Two smaller measurements fell out and both are now in §6.9. Padding costs **+6.4 % of bytes**
(75,622 → 80,497 for `21BD1`), so R-25 is free and there is no size trade to argue about. And the
decoy count is **not fixed** — 110 at the entry check, **125** hours later, same prefix, same 1,924
real rows — which is independent confirmation that gate line 2's row band could never have been met.

Two things the contract settled that the task list had not asked about. **The reuse grouping key
never crosses IPC in any form**, not even shortened into a group id the webview could colour rows
by: a hash of a short secret is a secret, and `shared_with` names the other members instead, which
is what the user needs and not what an attacker does. And **nothing in Phase 4 produces `expired`**
— the variant exists in `ItemStatus` and the view draws a group for it, and no requirement defines
a rotation date, so the group stays empty and the surface must not imply a check that is not
running. That is D-49's class of false promise, one screen further on, caught before it was drawn.

Phase 4 is **1/19**.

### 2026-08-15 (the sitting, and the gate down to one line)

The binaries were rebuilt first, and that was the whole reason the day worked: the ones sitting in
`target/release/` were from 2026-08-08 **12:11 and 10:55**, and D-70's profile landed at 18:13 that
day. Rows 20 and 21 describe surfaces that had never existed in any binary on this machine, so the
three rows the walk could not close were not a walking problem. `trustvault`, `trustvault-measure`
and `/tmp/reference.tvault` were rebuilt from `be278bc`, and the two binaries were checked to be
actually different — `webkit_web_view_get_inspector` appears once in the measure build and not at
all in the default one, so the walk had no inspector pane in its tab order (D-65).

**The keyboard audit is complete: 22 of 22 surfaces, 7 of 7 global rules, S-08 met.** Rows 20 and
21 passed on the rebuilt binary, including the clause the same day's fix had created — one Tab
leaves the popover instead of stepping through *Lock vault* into what sits behind it, which is
**finding 10 confirmed by a person rather than by a commit**. Row 21 passed on **both** open
paths, which is `Dialog.svelte`'s `isConnected` guard exercised on the path where the opener is
gone by the time the dialog closes; no other row reaches it.

**Row 12 closed by being re-worded, not by being conceded — D-73.** Its clause described a
generator dialog opening from inside the New-item dialog, and nothing in the product does that.
Third time this audit has produced that shape (D-64, D-67, now this), and the third time the call
was the author's. Re-wording was the smaller change and the safer one: a dialog over a dialog is
the one focus path `Dialog.svelte` has already been wrong about. The cost is written into the row
rather than left implicit — no length or set control while creating an item, and ⌘G is that path.

**The functional line ran.** Item created through the UI on the author's own vault, quit,
relaunched, unlocked, read back. It is an observation like G-B′'s fifth line, not a measurement,
and **nothing in CI reproduces it** — no CI job runs the application, which is the gap D-47 came
through.

**§10's twelfth box ticked with S-08**, on the same evidence, as its own wording said it would.
`MASTER.md` now records what the walk cost: four of the ten findings came from a person using the
application while three automated audits passed, and two of those four were not keyboard defects
at all — the pointer path was broken identically and nobody had noticed.

Phase 3 is **38/39**, gate **5 of 6**. The one task left is S-04's end-to-end number, which the
author reports having taken on `trustvault-measure` — it is **not recorded here until the figures
are**, because a criterion written as ≤ 50 ms p95 is met by a number or it is not met at all. The
one gate line left is the **7-day drive, and it has not started**.

**Phase 4 opened the same day, with that gate line still open — D-74.** The drive costs time
rather than work, so holding the phase shut for it would buy a week of idleness and no evidence.
The entry-check box for dependencies stays **unticked for the life of the phase**, which is what
makes the deviation visible to the next reader rather than a thing that has to be remembered.

**The entry check earned its place on the fourth box.** "External dependencies met — verify against
the live service, not against this document" was written at kickoff by somebody who could not have
known what it would find. HIBP is still free, still unauthenticated, and still honours
`Add-Padding` — measured, not assumed: 2034 rows with the header and 1924 without, the difference
being 110 zero-count decoys. And that measurement **falsified the phase's own gate line**, which
asks for responses of 800–1000 rows: 1924 of those rows are real hashes, so the band is impossible
rather than unmet, and no amount of correct implementation could ever have satisfied it. Found on
the day the phase opened, at a cost of one HTTP request, rather than at the gate with the client
already written to a number that does not exist. It is the fourth time in this project that
somebody has taken a criterion seriously enough to check it and found it wrong — D-64, D-67, D-73,
now this — and the re-wording is the author's, in the open questions above.

**Four tasks were added at the entry check** and all four come from Phase 3 rather than from the
roadmap: the contract before the command, a finding proven elided in `ipc_session.rs`, harness
fixtures added with the field rather than after it, and row 17 of the keyboard audit re-walked once
the Watchtower view shows real findings instead of fixtures. Phase 4 is **0/19**.

### 2026-08-14, later (the walk, and the three rows the record could not carry)

The author reported walking the audit with the pointer unplugged and everything passing. Nineteen
of the twenty-two boxes tick on that report, including the three re-walks the 2026-08-08 fixes had
re-opened — rows 3, 6 and 11 — and **no finding came back**, the first pass here that returned
none.

**Three rows are held, and the reason each is held is worth more than the tick would have been.**
Row 12's clause describes a dialog that does not exist (`NewItemDialog.svelte:124` fills the field
in place), so the row cannot pass as written and the re-wording is the author's call, the same
shape as D-64 and D-67. Rows 20 and 21 describe D-70's profile surface, and the only binaries on
this machine were built **2026-08-08 at 12:11 and 10:55** — the profile landed at 18:13 that day
and finding 10's fix on 2026-08-14, so no build here can have shown them. `/tmp/reference.tvault`
does not exist either, which is the same signal from the other end: S-04's number is still owed.

**What this costs is nothing and what it buys is the gate line.** S-08 is 100 % or it is not met,
so nineteen of twenty-two leaves the phase task and the gate line exactly where they were — 37/39,
gate 2 of 6. What it avoids is a ticked box that nobody could later tell from a considered one,
which is the failure mode the standard names in as many words. Three rows is a five-minute re-walk
on a fresh binary plus one wording decision, not a repeat of the sitting.

### 2026-08-14 (the script that found two defects, kept — and it found a third)

Opened with Phase 3 at 37/39 and both remaining tasks belonging to the author: the keyboard walk's
nineteen unwalked rows and S-04's end-to-end number, neither of which anything in this repository
can run. What was left for the repository was a decision it had been asked three times and never
answered — actions 12, 21 and 22 — and the answer is **D-71 and D-72**.

**The third audit exists because the first two structurally cannot ask its question.** `focus` and
`contrast` both reach their elements with `element.focus()`, and that succeeds on a
`tabindex="-1"` control exactly as it does on a real tab stop. So a control that has left the tab
order passes both audits while being unreachable by keyboard, which is not a hypothetical: it is
finding 6, and the script that caught it was thrown away the same day. `scripts/audits/taborder.js`
is that script kept, with the rules written down.

**It found a defect on its first full sweep, on a surface four days old.** The profile popover
(D-70) is `role="menu"` with four ordinary buttons inside it — four tab stops where ARIA defines a
menu as one, so Tab from *Settings* stepped to *Lock vault* and then out of the open popover into
whatever sits behind it. Nobody had reported it and nobody would have: with a pointer it is
perfect, and with a keyboard the arrows, Esc and Enter all work. What is broken is **leaving**.
Fixed in `Sidebar.svelte`'s pattern — one roving tab stop, arrows moving it — and **row 20 stays
unticked**, because the fix has not been walked and the row is what says whether it worked.

**Finding 10 is the first defect here against a specification rather than against a row.** Rows 6
and 12 both needed the author to decide what the surface should do; `role="menu"` had already
said. That distinction is also the audit's own boundary: it refuses to judge **how many** tab stops
a surface has, because thirteen consecutive sidebar rows is a defect against D-67 and an ordinary
navigation column anywhere else, and nothing in the DOM tells them apart. The count is printed
(`--stops`) for a person to read and fails nothing.

**The grouping rule was wrong twice before it was right**, and both errors are in the source
because they are the reason it looks the way it does. Climbing while every operable descendant
carries an explicit `tabindex` is the obvious rule: it **over-climbed** out of `role="radiogroup"`
and merged Settings' Theme with Interface size into one six-member group reported as broken, and
it **under-climbed** in the sidebar, whose thirteen rows share one roving index across three
`<ul>`s under a `<nav>` that also holds the footer button — so two of the three lists were
reported unreachable. Two false findings and one missed group on one screen. Boundaries fixed it:
a composite role is the author saying "this is one widget", a region is as far as peers can reach.

**Non-vacuity was proven the way this project proves it** — by breaking the rule on purpose.
Restoring finding 6's markup and deleting `ui_scale` from the fixture reproduces
`unreachable-group` on Interface size by name; marking a second option `0` reproduces
`ambiguous-group` on Auto-lock and Clear clipboard. Both reverted.

**And the audits are in CI (D-72).** Eight minutes, `browser-actions/setup-firefox`, three audits
over twenty scenarios in both themes, then the twenty-five screens rendered and uploaded as an
artifact. The full sweep after the fix is **120 surface-audits, no findings**, which is the
number the job will hold. The screenshot half is deliberately **not** a pixel comparison: a baseline taken on this
desktop fails against a runner's font rendering on the first run rather than on a regression, which
teaches people to ignore the job. What it does test is that every screen still renders, which is
the same class as the audits' "nothing to measure" and the class that has actually bitten. The job
has **not run on a runner yet** — it is on this branch, and the first PR is what proves it.

Phase 3 is **37/39**, gate **2 of 6** — unchanged, and correctly so: nothing here ticks a box. The
critical path is still the author's sitting, unchanged since the import surface landed.

### 2026-08-02

Kickoff. Read the Claude Design project `Brankas App Design`
(`7d8165c1-ec9a-4424-a50e-a24f8c2ac96f`) — `TrustVault App.dc.html`, a 1339-line interactive
prototype covering 15 surfaces, plus `TrustVault Logo.dc.html`. Confirmed the remote
`uploads/MASTER.md` is byte-identical to the local
`design-system/password-manager/MASTER.md`, so the design system has one source of truth and no
reconciliation was needed.

Two contradictions found in the prototype and resolved: sync (D-03), and the "Clears in 12s"
clipboard promise (now an open question rather than a silent assumption).

Ran the prior-art survey before proposing anything. Four findings changed the plan rather than
merely informing it:

- The **webview heap cannot be wiped** (`tauri-apps/tauri` discussion #10852). This is why the
  interface contract in `trustvault-requirements.md` limits IPC to one secret per command and why
  `copy_field` never returns a value — it is an architectural constraint discovered before any code
  was written, not a hardening pass bolted on later.
- **Clipboard ownership on Wayland/X11** belongs to the copying process, and clipboard managers
  cache secrets indefinitely. Turned into an open question against the Phase 2 gate.
- **HIBP response padding is opt-in**; without `Add-Padding: true` the response *size* leaks the
  queried prefix. Now R-25.
- **Windows OV certificates must live on an HSM** since June 2023. Moved the signing-account setup
  forward into Phase 3 so the paperwork queue does not block Phase 5.

Framing answered: Svelte 5 + TS + Vite; custom `.tvault` with Argon2id + XChaCha20-Poly1305; full
v1 scope (core + TOTP + Watchtower + multi-vault); all three platforms. Six-phase breakdown
proposed and confirmed. Document set generated. Repository initialized on `main`.

Toolchain checked: Rust 1.97.1 and Node 24.18.0 present; all five Tauri Linux system dependencies
were missing. The install needs interactive sudo, which the agent could not provide, so the user
ran it mid-session — `cargo tauri info` now reports webkit2gtk-4.1 2.52.3 and rsvg2 2.61.3 clean.

Scaffold built and verified in the same session. Green locally: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (3 tests),
`svelte-check` (0 errors over 4051 files), `prettier --check`, `vite build` (56.7 kB JS / 21.6 kB
gzip, 11.3 kB CSS, fonts emitted as local assets), and the N-02 boundary check — `trustvault-core`'s
entire dependency tree is serde, thiserror, zeroize and their proc-macros.

Three things bit during the build and are worth remembering:

- **Vite 8 dropped the bundled esbuild minifier.** `minify: 'esbuild'` now fails with a missing
  package; the minifier is `oxc`.
- **`svelte-check` cannot use TypeScript 7** (peer range is `^5 || ^6`), so the toolchain is pinned
  to TS 5.9.3 even though 7.0.2 is current. Revisit when svelte-check widens its range.
- **The snap-terminal crash** (D-16), which cost the most time and looked exactly like a broken
  build until the binary was run under a scrubbed environment and started fine.

The app now compiles, launches, and serves the specimen. What is *not* verified: that the window
renders correctly and that both themes switch — there is no screenshot tool on this machine, so the
visual half of the gate needs human eyes. CI has never executed either, because the repository has
no remote yet. Neither is ticked.

### 2026-08-02 (later)

Repository published at `github.com/shoelfikar/trustvault` (public, D-17) and CI ran for the first
time. **All three platform builds passed on the first attempt** — Linux, macOS 15, and Windows 2025
— which was the sequencing risk most likely to bite, since none of those runners had ever executed.

The audit job failed, for two unrelated reasons that looked like one:

1. `rustsec/audit-check` could not create its check run — `Resource not accessible by integration`.
   The default `GITHUB_TOKEN` is read-only, so the job needs `permissions: checks: write`. This is a
   reporting failure, not a finding, and it masked the real result.
2. Three genuine vulnerabilities, all denial-of-service, all transitive through Tauri:
   RUSTSEC-2026-0009 (`time`, stack exhaustion) and RUSTSEC-2026-0194/0195 (`quick-xml`, quadratic
   parse and unbounded allocation, reached via `plist`).

The interesting part is why they were not simply patched away: `cargo update` refused to move,
reporting "Locking 0 packages to latest Rust 1.85 compatible versions". The workspace `rust-version`
had been set to 1.85 as a conservative default, and the patched releases need 1.88 — so a setting
chosen for caution was holding three security fixes out of the build. Raised to 1.88 (D-18), after
which `time` went 0.3.45 → 0.3.55 and `quick-xml` 0.38.4 → 0.41.0. Clippy and the tests still pass.

The remaining 17 findings — 16 unmaintained gtk-rs GTK3 crates plus `glib`'s `VariantStrIter`
unsoundness — arrive through Tauri's Linux backend and cannot be fixed here. They are now ignored
individually in `.cargo/audit.toml` with a reason and a retirement condition each (D-19), rather
than left to make the audit job permanently red.

Phase 0 gate passed later the same day. The audit fixes went green on run 30747639101 — all seven
jobs, including the three platform builds. The last open task, "no network font requests", was
closed by checking the built bundle rather than a devtools session: every `@font-face` source
resolves to a local `/assets/*.woff2`, there is no `@import` or font-CDN reference anywhere in
`dist/`, and the CSP pins `font-src 'self'`. That is reproducible in CI later, which an eyeballed
network tab is not.

Phase 0 is closed at 23/23. **Phase 1 is not open** — its entry check has not been run, and running
it is the first next action.

Audited `.gitignore` against `90-MOC/Gitignore Standard.md`. Three real findings, none of them a
wrong pattern:

- The repo's `.gitignore` carried `.DS_Store`, `Thumbs.db`, `.idea/`, and `*.swp`. Per the
  standard those are a statement about somebody's editor, not about this project, and belong in
  `~/.config/git/ignore` — which did not exist and was not configured. Created it, set
  `core.excludesFile`, and removed the lines from the repo.
- `!.env.example` had no comment. The standard requires a reason on every `!` line, because an
  exception without one gets deleted by the next person tidying up and the effect surfaces much
  later.
- `*.cer` was grouped with secrets. A `.cer` is a public certificate; the line was dropped and
  `*.pem` and `*.key` added in its place, which are the ones that actually matter.

Two things were checked and found **correct**, contrary to first impressions. The KAT-vector
exception `!crates/trustvault-core/tests/vectors/*.tvault` works — `git check-ignore -v` prints
negated matches too, so its output initially read as a failure; `git add --dry-run` shows the
vectors are committable while `personal.tvault` is refused. And nothing derivative was tracked by
accident: `git status --ignored` lists only `dist/`, `node_modules/`, `target/`, and
`src-tauri/gen/`.

The icon question is written up as D-20. It is the one place this repo knowingly departs from the
standard's default, so the reason is recorded rather than left to be rediscovered.

### 2026-08-02 (Phase 1)

Phase 1 opened and, apart from one measurement, closed in the same session.

**Entry check passed, all six boxes.** The dependency was read from the gates table rather than
from ticked boxes, as the rule requires: Phase 0 passed on run 30747639101. Two things the plan had
not anticipated were added to the task list rather than done silently — a CI gate on nonce
discipline, which R-06 asks for and Phase 0 never shipped, and a dedicated Cargo profile for
coverage runs.

**`docs/vault-format.md` was written before any cryptography**, and it earned its keep immediately.
Writing §4 forced the question of what each of the three seals authenticates, and the answer written
down was wrong: both salts as shared associated data for both key wraps. The test suite failed
within the hour with an exact symptom — changing the master password invalidated the *recovery*
wrap, which cannot be re-wrapped at that moment because the user is not holding their recovery code.
Narrowed to the parameter block, header bytes 0..20 (D-25). Nothing is lost: a salt is already a KDF
input, so editing one derives a wrong key, and both salts remain covered by the body tag. The
document had made the claim explicit enough to be testable, which is the entire argument for writing
it first.

**The Argon2id open question is closed by measurement** (22 parameter sets, table in
`docs/vault-format.md` §3.2). 256 MiB / t=3 / p=1 gives 511 ms — over S-03's 500 ms floor by 2 %,
and that thin margin is the useful finding. "Per platform" turned out to be the wrong shape for the
answer: the variable is the machine, not the operating system. `KdfParams::calibrate` measures the
machine at vault creation and writes what it found into the header, so the constant is a starting
point rather than a promise. `p_cost` stays at 1 because this implementation does not thread
Argon2's lanes, and raising `p` without threads costs the same wall clock while reducing
memory-hardness per lane.

Four dependency decisions, all with prior art before adoption (D-21…D-24). The one worth
remembering is D-23: **no `rand` crate anywhere in the core.** R-06 forbids counter-derived nonces,
and the durable way to keep that true is to have nothing in the tree that could become a counter —
`getrandom::fill` has no seeding and no `SeedableRng` to reach for in a hurry. A CI grep enforces
both halves.

Evidence produced for the gate:

- **R-04**, byte mutation: every *bit* of every byte of a fixture vault flipped, 2 664 mutations,
  all rejected. Plus truncation at every length and appended bytes at every length.
- **R-03**, timing equivalence: 41 interleaved samples per arm, median difference **0.09 %** against
  a 5 % tolerance. The test retries up to three independent rounds, because a shared CI runner
  stealing a core mid-sample is noise, not a finding — but a real difference reproduces in every
  round and all three must fail for the test to fail.
- **R-05**, 100 injected kills: real `SIGKILL` to a child process saving in a loop, 100/100 left a
  complete readable vault at the destination. What the harness **cannot** do is land a randomly
  timed kill between `File::create` and `rename`: on the tmpfs a temp directory lives on, that
  window is a memcpy plus a no-op fsync, microseconds out of a save measured in tens of
  milliseconds. Asserting it would have produced a flaky test rather than a stronger one, so the
  rename property is proven deterministically instead — the destination's inode changes across a
  save, which it cannot do if the file were written in place.
- **R-01**, round trip: 10 000 generated vaults, including empty strings, astral-plane Unicode,
  combining marks, right-to-left text, and embedded NULs.
- **N-09**: unknown keys injected at vault, item, and field level survive a full load/save/load
  cycle, including nested ones.
- **KAT vectors**: three committed `.tvault` files, each with a JSON manifest holding the password,
  the recovery code, the header parameters, and the exact plaintext. They are the only artefact a
  refactor cannot argue with, because everything else tests the implementation against itself.

Three things bit during the work and are worth remembering:

- **`cargo llvm-cov` blocks on an invisible rustup prompt.** It asks to install
  `llvm-tools-preview` and waits on stdin; through a pipe the prompt never flushes, so the command
  looks hung with no output and 0 % CPU for as long as you leave it. Install the component
  explicitly first. CI does this via the toolchain action and never sees it.
- **Unoptimized cryptography makes the suite unrunnable.** The mutation test ran 47 s before
  `[profile.dev.package."*"] opt-level = 2` and 7 s after. Under coverage instrumentation the
  unoptimized version is worse still, which is what the separate `coverage` profile is for — and it
  forces `panic = "unwind"`, because proptest shrinks a failing case by catching panics.
- **Clippy's `allow-unwrap-in-tests` does not reach integration tests.** An integration test is its
  own crate with no `#[cfg(test)]` module, so the Tier-1 lints in `Cargo.toml` apply at full force
  and every `unwrap` in a fixture is a build error. Lifted per file, with the reason at the top of
  each.

Not yet done, and the reason the Phase 1 gate is **not ticked**: the coverage figure has not been
read. `rustup component add llvm-tools-preview` is still downloading on this machine. Every other
gate criterion has evidence behind it. The CI job that enforces `--fail-under-lines 90` is committed
and will produce the number independently.

### 2026-08-03

**Phase 1 gate passed.** The one outstanding piece of evidence was the coverage figure; the
`llvm-tools-preview` download that blocked it had completed, so the measurement was simply run.

First measurement: **98.39 % lines, 96.10 % regions, 98.06 % functions** — over N-03's 90 % floor,
and `--fail-under-lines 90` exits 0. So the gate was already satisfiable without touching the code.
The 18 uncovered lines were read anyway rather than accepted on the strength of the total, and they
split into two unrelated kinds:

- **Seven lines of untested public API** — `Default for KdfParams`, `From<String> for SecretString`,
  and `Vault::body()`. Each carries a comment making a real claim (the default is the *measured*
  constant; `From<String>` moves rather than copies so no un-zeroized buffer is left behind;
  `body()` is the shape that must never reach a Tauri command), and none of those claims was
  attached to anything executed. Three tests added, one per claim.
- **Nine lines of allocation-failure fallback** in `KdfParams::calibrate_to` — the path taken when
  Argon2 will not allocate even at the floor. Reaching it needs a failing allocator, and there is no
  honest way to fake it from a test. Left uncovered and written into the gate as such, with the
  exact line numbers, so it reads as a decision rather than an oversight.

After the three tests: **99.21 % lines, 96.64 % regions, 100 % functions.** Every uncovered line in
the crate is now that single allocation-failure region.

The useful part was not the number, which passed on the first run. It was that "≥ 90 %" would have
let seven lines of unexercised security-relevant API through without anyone looking, because the
total was comfortable. The threshold catches a crate with no tests; it does not catch this. Reading
the uncovered list is the step that does, and it costs one command.

Phase 1 closed at **37 of 37**, gate ticked. Local verification green: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -D warnings`, and 59 unit plus 18 integration tests. Phase 2
is **not open** — its entry check has not been run, and running it is the second next action.

One caveat carried into the next session: all of this is local evidence. CI has never executed the
coverage job, because the whole of Phase 1 was uncommitted until now.

### 2026-08-03 (icons)

The design's icon folder was moved into the frontend at the author's direction: `icons/ui/` (30
glyphs) and `icons/brand/` (13 marks) now live under `src/lib/icons/`, with `Icon.svelte` inlining
the set through `import.meta.glob`. Inlined rather than served as `<img>` because an `<img>` cannot
inherit `currentColor`, and `MASTER.md` §8 specifies an icon that is `--fg-muted` at rest, `--fg` on
hover and `--accent` when active.

The instruction had a consequence worth naming rather than absorbing quietly: **the shipped set is
not Lucide**, so it departs from §8 (D-28). §8's *one set, no mixing* clause is what decided the
shape of the change — `@lucide/svelte` was removed rather than kept alongside, because keeping both
would have broken §8 twice instead of once. The gap this creates is already visible: the set has no
`palette` glyph, so the workbench theme toggle now uses `refresh`, which names the action instead of
the subject. Every future gap has the same two answers, draw it or re-word the control, and neither
is free.

`scripts/make-icon.py` is deleted (D-29). It re-drew the mark in Pillow to make `icon-source.png`
reproducible, which meant the repo held two independent definitions of one piece of artwork with
nothing keeping them in step. `tauri icon` accepts an SVG directly, so the brand file is now the
source and the intermediate PNG is gone. `src-tauri/icons/` was regenerated and stays committed —
D-20's argument for that is untouched.

One thing bit, and it was not the icons. `npm run fmt` reformatted two **known-answer test vector
manifests** — `v1-all-kinds.json` and `v1-unicode.json` — because nothing had ever excluded them
from Prettier. It is harmless to JSON parsing and would have been committed without comment, which
is the problem: those files are Phase 1's evidence, and the entire reason they are worth having is
that nothing but a deliberate regeneration ever touches them. Reverted, and
`crates/trustvault-core/tests/vectors/` is now in `.prettierignore` along with the icon sources,
whose single-line path data Prettier reflows into something no longer diffable against the design.

Verified: `svelte-check` 0 errors over 283 files, `prettier --check` clean, `vite build` green, and
`cargo check -p trustvault` still compiles against the regenerated icon set. Bundle went 56.7 → 58.5
kB raw but 21.6 → 20.3 kB gzipped — the 30 inlined glyphs cost less than the two Lucide components
they replaced.

### 2026-08-03 (snap loader fault, second occurrence)

`npm run dev:app` still died with `undefined symbol: __libc_pthread_init, version GLIBC_PRIVATE`,
which D-16's script was written to prevent. Diagnosed rather than patched again, and the mechanism
D-16 recorded turned out to be the wrong one.

It is not `LD_LIBRARY_PATH`. That variable is empty in the editor's terminal, and it could not be
otherwise — pointing it at core20 breaks `bash` itself, verified by doing it. The real path is
`RPATH`: `readelf -d` on `/snap/code/254/.../libpixbufloader-png.so` and `im-thai.so` shows
`RPATH: [$ORIGIN/../../..:/snap/core20/current/lib/x86_64-linux-gnu]` baked into the objects.
`RPATH` outranks `LD_LIBRARY_PATH` and cannot be overridden from the environment at all. So when
`GDK_PIXBUF_MODULE_FILE`, `GTK_IM_MODULE_FILE`, or `XDG_DATA_DIRS` point GTK at a snap module, GTK
dlopens it, that object drags core20's glibc 2.31 into a process already holding the host's 2.43,
and the loader fails. It fails on window creation, not at `exec` — which is why the binary itself
`ldd`s clean and why the fault looks like an application bug.

`scripts/dev.sh` now builds the child environment from an allowlist (`env -i` plus named variables)
instead of unsetting a list of known-bad names (D-30). The old list had three live gaps —
`XDG_DATA_DIRS`, `XDG_DATA_HOME`, `GST_PLUGIN_SYSTEM_PATH` — and no list can be complete against
whatever the next snap revision exports.

Verified: probed the script under a simulated polluted terminal and no `/snap` path survives into
the child; then ran `npm run dev:app` end to end — vite on :1420, `cargo run` green, and
`target/debug/trustvault` alive with no loader error.

### 2026-08-04 (the shell)

Phase 2's task list is **finished, 26 of 26**. Its gate is not, and one gate line turned out to be
unmeetable by this phase's own code.

**The gate asks to "read an item" and Phase 2 cannot create one.** Found while building the detail
pane rather than by reading the gate: `add_item` and `update_item` are Phase 3 by the roadmap's own
split, and this phase deliberately shipped no mutation commands, so a vault created through
onboarding is empty and there is nothing to read. That is a plan defect — the gate text was written
at kickoff, against a phase boundary drawn later. It is now an open question with three named ways
out rather than something to be improvised on gate day. Worth noticing that the entry check *did*
its job on the other four gate lines and missed this one, because "still measurable as written"
reads the sentence, not what the phase will be able to put in front of it.

`SecretField` is where the whole IPC design finally pays off, and two of its properties are worth
recording because they are easy to undo. The remask countdown drawn on screen is **cosmetic** — the
host owns the real timer and pushes `field-remasked`, so a paused JS timer or a frozen tab cannot
keep a secret visible. And `copy` has no code path that could capture a value, because `copy_field`
is not offered one; the absence is the requirement.

Three design rules produced code rather than decoration:

- The item row's brass left bar is **always present and transparent**, so selecting a row does not
  shift its text by 2px. A layout that moves on selection reads as a bug.
- Focus is a 1px inset ring and selection is a fill, because §7 requires a keyboard user to see
  where they are and what is open at the same time.
- `unknown` renders as **nothing** rather than a fourth status colour. It is the state every item
  starts in, so drawing it would put a pip on every row of a vault nobody has scanned — which is
  how a status indicator stops meaning anything.

No brass anywhere means "good": the Watchtower badge is `--warn`, the strength meter runs
danger/warn/ok. §2 reserves the accent for brand and interaction, and a status surface is where
that rule is most often broken.

`docs/icon-gaps.md` now exists, and the worst gap is real rather than cosmetic: `ssh_key` and
`api_key` both fall back to `terminal`, so **two of the seven item types are indistinguishable in
the list**. `key` is already spent on `login`. That is a glyph somebody has to draw, and D-28's
"no upstream" cost is now a concrete line item rather than a warning.

Two tasks were ticked with their scope narrowed rather than silently: the sidebar has **no profile
footer** (it belongs to multi-vault switching, Phase 3, and an avatar with nothing behind it is
decoration), and pane resizing is **pointer-only** (keyboard resize is a Phase 3 keyboard-audit
item, and claiming it now would put a tick against S-08 that nothing earned).

Verified: `svelte-check` 0 errors over 298 files, prettier clean, `vite build` green at 78.43 kB /
26.63 kB gzipped, `cargo clippy --workspace --all-targets` silent, all 12 test suites pass, and the
app boots with no errors or panics. **Not verified:** the item list, the detail pane, and the
secret field against real data — a fresh vault is empty, which is the gate defect above wearing a
different hat. The data path underneath them *is* tested, by the IPC harness, which drives
`list_items`, `get_item`, and `reveal_field` against a fixture vault holding a real item.

### 2026-08-04 (onboarding, lock, recovery)

Five screens' worth of work against a contract that was already fixed, which is the whole argument
for having written it first: this session wrote UI, not UI *and* a boundary at the same time.

`src/lib/ipc.ts` is the only file in the frontend that calls `invoke`. That is not tidiness — it is
the one file a reviewer has to read to answer "what can reach plaintext from here", and a stray
`invoke` anywhere else makes that answer wrong. The three sanctioned calls are grouped and labelled
at the bottom of it, so the budget of three is visible from both sides of the boundary now.

Routing is driven by `vault_status`. There is **no local `unlocked` boolean anywhere in the
frontend** — the same property the Rust harness checks from its side, expressed here as an absence.
A reload lands back on whatever the host says, and for a locked vault that is the lock screen.

**zxcvbn contradicted D-12's own example, and the test now says so.** D-12 justified the crate with
`Jakarta2019!` — the password entropy-only scoring calls strong and zxcvbn supposedly catches.
Measured while writing the tests: it scores **3 / Strong** with the default dictionaries, because
"Jakarta" is not in zxcvbn's English-centric frequency lists. The decision stands — zxcvbn is still
right for the crack-time phrasing the design uses, and it does catch `password` and `hunter2` — but
its example was aspirational. What closes the actual gap is the `inputs` parameter, which feeds the
vault name in as context; told the vault is called "Jakarta", the same password scores far lower.
Both halves are pinned by a test so nobody re-derives the claim from the decision log and believes
it.

Two commands were added, each with the contract updated **before** the code, which is now the
habit the harness enforces rather than a good intention:

- `score_password`, in the host rather than the webview. zxcvbn's dictionaries are a few hundred
  kilobytes, and S-01 budgets 800 ms for cold start; a dictionary the frontend parses before it can
  draw is paid on every launch to serve a screen the user sees once.
- `default_vault_path`, which is deliberately **not** a native file picker. A picker means
  `tauri-plugin-dialog`, and the manifest's standing rule is that a plugin arrives when a
  requirement needs one and not before — a plugin is widened attack surface in a process holding
  decrypted secrets. R-08 asks for "name & location", which a resolved default and an editable path
  satisfies. Revisit if the typed path turns out to be what users get wrong.

Two product calls worth recording. **Weak passwords are warned about, not blocked** — a creation
flow that refuses the password someone chose teaches them to append "1!", which is the same
password with a suffix the attacker also knows about. And **unlocking with a recovery kit spends
it**, so the flow issues a fresh one and shows it before it can be left; otherwise a user recovers
once and has no way back in the next time.

What is **not** verified: how any of it looks. `svelte-check` is clean over 291 files, `prettier`
passes, `vite build` is green at 62.97 kB / 21.84 kB gzipped, and the app boots — vite resolves
`@tauri-apps/api` in the webview, so the frontend loaded and the window opened with no loader
error. But there is still no screenshot tool on this machine, the same gap Phase 0's gate hit, so
every visual claim in this session is a claim about the code rather than about the pixels.

### 2026-08-04 (Phase 2 opens)

**CI reproduced the Phase 1 gate.** The caveat that gate closed with was that every piece of its
evidence was local and the coverage job had never executed on a runner. The workflow only triggers
on pushes to `main` and on pull requests, so the whole of Phase 1 sitting on `development` had never
built in CI at all — it was not that the coverage job was skipped, it was that no run existed.
Dispatched manually against `development`: run 30868118687, all nine jobs green, including the three
platform builds, the N-02 isolation check, and the R-06 nonce grep.

Coverage on the runner: **99.12 % lines, 96.59 % regions, 100 % functions**, against 99.21 / 96.64 /
100 measured locally. One extra uncovered line on the runner, not chased — both figures are eleven
points clear of N-03's floor and the difference is a single line, but it is written down rather than
rounded away, because "CI matches local" is the kind of claim that should be checkable later.

**The audit-log open question is closed as D-31**, which was the one thing standing between Phase 1's
gate and Phase 2's entry check. The prior-art search returned nothing to copy and that was the useful
result: 1Password's audit log is server-side and Business-only, KeePassXC has no reveal log at all
and the issue asking for one (#5573) is still open. So the shape is being invented here rather than
adopted, which is the argument for the conservative half of the decision — the setting is **off by
default**, against `MASTER.md`'s unconditional wording, and that departure is recorded rather than
absorbed into the Settings copy.

The location was never really in doubt once §9 of the format spec was re-read: a new body key is
already documented as *not* a format change, so the log goes behind the same seal as the secrets with
no `format_version` bump and no KAT regeneration. What took the thinking was **when it is written**.
Flushing on every reveal is durable to the entry and costs a full re-encrypt, fsync, and rename on an
operation the user experiences as a read — the vault's mtime and inode would change every time a
password is looked at. Buffered and flushed on save loses the tail on a crash, which is acceptable
once the log's actual reader is named: the vault's owner reviewing their own habits. It is not
tamper-evidence and never could be, because anyone holding the master key can rewrite it.

Retention went to a **count cap alone**, 1000 entries, at the author's direction. The rejected age
cap is not free and the gap is in the decision log rather than left implicit: a light user's 1000
entries may reach back years, so a cracked vault exposes a longer history of habits than a 90-day
cap would have allowed.

**Phase 2 entry check passed, all six boxes**, and the phase is open. Three findings changed
something rather than merely being recorded:

- **Three of the eleven requirements are already partly satisfied.** R-07's core half — unwrapping
  without the master password — shipped in Phase 1, so this phase owes only the kit UI and the
  recovery entry flow. R-28's OS-following half is already in `tokens.css:144`; what is owed is the
  persisted override. N-08's duration collapse is already in `tokens.css:192`; what is owed is not
  bypassing it, and the unlock scrim is the one place tempted to. Phase 2's share is narrower than
  the kickoff task list implied, which is exactly what an entry check is for.
- **The task list grew from 22 to 25.** `docs/ipc-contract.md` is now written *before* the command
  layer, on Phase 1's evidence that a document written first makes a claim testable enough to be
  proven wrong within the hour (D-25); after the code it would only describe what was built. The
  reveal log split into a core task and a command task, because D-31 makes the storage half
  `trustvault-core` work. And `docs/icon-gaps.md` was added, because D-28 left the project with a
  30-glyph set and no upstream, the first gap surfaced within a day, and this phase draws far more
  surfaces than Phase 0 did.
- **Two gate lines were nearly collapsed into one.** "Clipboard clears within the configured window"
  and "the clipboard-manager question is answered" measure different things: the first is a stopwatch
  against TrustVault's own clear, the second is whether a manager kept its own copy regardless, which
  no amount of clearing affects. A pass on the first says nothing about the second. Recorded as a
  clarification in the entry check rather than an edit to the gate.

`feature/phase-2-shell-unlock` was cut from `development` after the entry check was recorded, not
before — the order matters, because a branch cut first is a phase opened by habit rather than by
check.

**`docs/ipc-contract.md` written**, the first task, and it earned its keep the way Phase 1's format
spec did: four things surfaced that the prose contract in `trustvault-requirements.md` had left
implicit, and three of them are not things the code would have gotten right by accident.

- **The lock screen cannot show the vault's name.** The name lives inside the sealed body, so while
  the vault is locked there is nothing to read. `vault_status` returns the file stem instead, and
  the contract says the lock screen must not imply otherwise. Nobody would have noticed this until
  the lock screen was built against an unlocked test vault.
- **The mask must not be the secret's real length** — D-32. Writing out `FieldSummary` made the
  obvious implementation visible: `"•".repeat(value.len())` puts the exact length of every password
  in the vault into an unwipeable heap, for every item, with no user action and nothing revealed.
  Fixed at 12.
- **`unlock_recovery_kit` is a *sanctioned* command and `unlock` is not**, which is not obvious from
  either name. Unlocking with a kit means the kit has been spent, so the flow issues a fresh code on
  the spot — and that fresh code is a secret travelling outbound. Counted in the budget of three,
  where it would otherwise have been smuggled in as "part of the unlock flow".
- **`history` has no command at all in Phase 2.** It holds previous values of secret fields, so a
  user's last five passwords for one site is a worse leak than any single one of them. Excluded from
  both `ItemSummary` and `ItemDetail` rather than elided, because a field that is present and masked
  invites someone to add the reveal later.

Two things the contract could not settle and did not pretend to. **Where settings live** is now an
open question: `theme` must be readable while locked so it cannot sit in the sealed body,
`audit_log_enabled` arguably should sit with the log it controls, and none of the four values is
secret. The command shape was written so it does not depend on the answer, which is the reason the
contract did not have to wait. And **R-10's acceptance criterion is unmeetable as written** — it
names a *core* test for a rule the core cannot see, because N-02 forbids `trustvault-core` from
depending on Tauri. The enforceable version is `src-tauri/tests/ipc_audit.rs`, already a task; the
requirement text needs correcting before the gate inherits an impossible criterion.

The contract also names what it costs to add a fourth sanctioned command, because Phase 3 will want
one: `generate_password` returns a secret. That is a decision log entry, not a patch, and the
alternative — generating straight into the clipboard, where `copy_field` already proves the pattern
— is worth considering first.

**The core's audit log, the command layer, and the audit harness all landed the same day.** Phase 2
is at 11 of 25; everything remaining is frontend.

The audit log went in as `docs/vault-format.md` §9 promised it could — a new body key, no
`format_version` bump, no vector regeneration. The detail that made that true is
`skip_serializing_if`: an empty log writes **no key at all**, so a vault that has never recorded a
reveal encodes exactly as it did before the field existed. Without it, every known-answer vector
would have needed regenerating for a feature none of them uses. The test checks the plaintext CBOR
rather than the sealed bytes, because searching ciphertext for a key name passes whatever the
encoding does.

`Vault::reveal_field` takes the audit flag itself rather than leaving the command layer to remember
— a boundary that *could* read a value without recording it is one that eventually does. The same
reasoning made `copy_field` audited: a copy that went unlogged while a reveal was logged would make
the log worse than useless, because it would look complete.

Three things in the command layer are worth remembering:

- **The remask timer runs in the host, not the frontend.** A frontend timer is cleared by a reload,
  and a reload must not extend a reveal. A generation counter, bumped on every lock and unlock,
  stops a timer that fires late from emitting an event about a vault that is no longer open.
- **R-09's third trigger is not what the requirement implies.** There is no portable OS sleep
  notification — logind over D-Bus, `NSWorkspaceWillSleepNotification`, and `WM_POWERBROADCAST` are
  three platform integrations, two untestable on this machine. `autolock.rs` uses a wall-clock jump
  detector instead, and the gap is real: it detects sleep on **wake**, not before it, so a machine
  that suspends with the vault open holds the master key in RAM while it sleeps. Written into the
  module and into the task, because that belongs in the gate evidence rather than in a comment
  claiming R-09 is met.
- **`arboard`'s clipboard value is kept alive for the life of the process.** On X11 and Wayland the
  copying process *serves* the clipboard; dropping the handle at the end of the command would make
  the copied password silently unpastable. D-11 chose the crate for documenting this rather than
  hiding it, and this is the day that paid off.

**The harness caught drift on its first run**, which is the best possible outcome for a test written
the same day as the thing it checks. `create_vault`'s signature was wrapped across three lines in
the contract, so the document could not be parsed against the `generate_handler!` list — the
contract was reshaped, and the check that compares the two sets now passes. §9 also gained a table
of what is automated and what is not, because check 5 (a scripted whole-shell session) needs a shell
that does not exist yet, and a check that quietly does not run is worse than one documented as not
running.

Settings storage is closed as D-33 rather than deferred: the contract's shape did not depend on it,
but the code did, and an app that forgets your theme every launch is a thing the author would fix
in a day and then never write down.

### 2026-08-04 (UI parity with the prototype)

Directed by the author: bring the app's UI to the prototype exactly, minus the prototype's own
demo chrome — the desk, the window frame, the Onboarding/Lock/App flow switcher and the theme
toggle in its header, which are how the *prototype* is driven and not part of the product. Asked
where the boundary sat for surfaces Phase 2 has no command behind; the answer was **all of them,
with dummy data where needed** (D-36).

**The prototype was read properly for the first time.** `TrustVault App.html` is a bundled
artifact: 393 lines, two of which are 285 kB and 153 kB. The second is a JSON-encoded template,
and decoding it yields 1 582 lines of the design's own markup with every pixel value inline, plus
the component's state and data at the bottom. Everything below was measured off that rather than
inferred from screenshots, which is why the geometry is exact and not approximately right.

Fifteen surfaces are now drawn. Seven existed and were rebuilt against the real values
(onboarding, lock, recovery, titlebar, sidebar, item list, detail pane); eight are new (Watchtower,
Settings, command palette, generator, New item, delete confirm, vault switcher, Trash). Nine new
components sit under them, of which `Dialog.svelte` earns its place twice: it owns Esc-to-close and
a focus trap, so §7's "full keyboard operation" survives a modal without the component library D-09
rejected.

**The biggest single finding was not a design one.** `global.css` had no element reset beyond
`box-sizing`, so every `<p>`, `<h1>` and `<ul>` in the app was carrying the user agent's margins and
`<ul>` was carrying bullets. It had been that way since Phase 0 and nobody could have seen it,
because *nobody had ever looked at the rendered app* — the gap Phase 0's gate hit and every session
since has repeated. It is fixed, and the fix is the argument for the next paragraph.

**There is now a way to see the UI on this machine.** The blocker was never a screenshot tool; it
was that the app needs a Tauri host to render anything. Headless Firefox screenshots
`dist/index.html` fine once `window.__TAURI_INTERNALS__` is stubbed with a fake `invoke` — the
frontend cannot tell the difference, because `src/lib/ipc.ts` is the only file that calls it. Eleven
screens were captured in both themes against fixture data. Two caveats worth carrying: Firefox here
is a snap and cannot write outside `$HOME`, and it screenshots at the load event, so dialog
enter-animations are caught mid-fade unless the harness disables them. The harness is **not
committed yet**; that is next action 2.

Four things it caught that review would not have:

- The element-reset defect above.
- `direction: rtl` used to truncate a long path from the left prints `home/…/personal.tvault/` —
  the leading slash moves to the end and the path shown does not exist. Removed.
- The detail pane opened empty on every launch. The prototype selects the first row; it is the
  right default, and a pane whose first words are always "select an item" spends the app's most
  valuable pixels on an instruction.
- The strength meter's empty segments are unreadable at `--fg-subtle` (D-34b).

Four deviations are logged rather than absorbed (D-34, D-35, D-36, D-37). The one worth repeating
here is **D-37**: the generator is drawn in full and its copy paths are closed, because the
clipboard clear that makes a copy safe is scheduled by Rust inside `copy_field`. A copy from that
dialog would leave a password in the clipboard that nothing ever clears — a weaker promise than the
chip on the detail pane already makes. The preview is minted with `crypto.getRandomValues` and
rejection sampling rather than the prototype's `Math.random`, so there is nothing here that could be
promoted into the real generator by accident.

Three of the prototype's own fictions were replaced with the truth rather than copied. The sidebar's
profile footer becomes the **vault** footer — there is no account and no sync (D-03), so an avatar
and an email would have been three invented fields; it carries the vault's initials, name, file and
item count and opens the switcher. Settings' Profile card becomes a **Vault** card the same way, and
its three facts (items, "Offline — this device only", the file) are all readable. The detail pane's
History column says the previous values stay sealed instead of counting versions, because
`docs/ipc-contract.md` §6.1 excludes history from `ItemDetail` on purpose. "or use Touch ID" is
absent rather than disabled: there is no biometric path in this build and a greyed-out one still
implies there will be.

Verified: `svelte-check` 0 errors / 0 warnings over 314 files, `prettier --check` clean,
`vite build` green at 125.13 kB / 39.88 kB gzipped (up from 78.43 / 26.63 — eight new surfaces),
and eleven screens rendered and read in both themes. **Not verified:** any of it against a real
vault, which is the "read an item" gate defect wearing its third hat; and none of it inside the
actual Tauri window, only in the same webview engine.

### 2026-08-05 (four gate lines, and what the harness found on the way)

The session's job was the Phase 2 gate. Four of its five lines now have measured evidence; the
fifth is the functional one and it needs hands on the packaged app, which is next action 1.

**Two of the three blockers were decisions, not work.** The author chose to re-word the functional
gate line and move "read an item" to Phase 3 (D-38), and R-10's acceptance criterion was corrected
to name the test that can actually enforce it (D-39). Both are cheap on the day and expensive on
gate day, which is the whole argument for having found them a session early. The risk D-38 carries
is written into the phase document rather than absorbed into a tick: G-B′ can now pass without a
human having watched a secret cross IPC in the running app.

**The instrumented session harness caught a real defect on its first run**, which is now the second
time that has happened — the IPC audit harness did the same on 2026-08-04. `tests/ipc_session.rs`
scripts a whole shell session against a real file and reads the transcript rather than the code,
and step four of the script printed `vault_status -> no_vault` where a relaunch belonged. **The app
forgot its vault on every quit.** The host keeps nothing across a launch, so the frontend routed to
onboarding — a user who created a vault yesterday was shown the create-a-vault flow today, with no
way back to their own file, because "Open vault file…" is drawn-but-inert (D-36). Fixed as D-40: the
path is remembered in the settings file and restored at start-up, only the path and never a key, so
the state a relaunch produces is `locked`.

Worth noticing *why* the defect survived until now. Every earlier check was shape-level — does this
response carry a secret, does this command refuse while locked — and each one was correct in
isolation. The bug lived in the *sequence*: nothing had ever asked what the fourth call returns
after the first three. It also explains the shape of the fix's own trap, which the tests pin:
`Settings` deserializes with defaults, so a frontend that does not know about `last_vault_path`
sends it absent, serde reads absent as `None`, and a theme change would erase the vault. That is
what `merge_incoming` is for, and why it is a named function with a test rather than a line inside
a closure.

**The clipboard-manager question is answered, and the answer is no** (D-41). GPaste 45.3 on
GNOME/Wayland with `track-changes` on recorded the copied value and still held it after TrustVault
cleared the clipboard. The finding underneath it is the more embarrassing one: the
`x-kde-passwordManagerHint` was **never being sent**. D-11 chose `arboard` over the thinner wrappers
precisely because it exposes that hint, and `clipboard::set` called plain `set_text`. It sends the
hint now, and GPaste ignores it anyway — so the decision's *reasoning* was sound and its *benefit*
was zero, which is only knowable by going and looking. The chip reads "TrustVault clears its copy in
*n*s"; `tests/clipboard_manager.rs` fails if a clear ever does reach a manager's history, which is
the day the wording may be strengthened.

Measured, rather than asserted:

- **Clipboard clear**: 1.000180 s against a 1 s window — 180 µs of overshoot against S-11's 200 ms
  budget. The test also reads the clipboard back, because a stopwatch alone passes for a
  `clear_after` that sleeps and does nothing.
- **Auto-lock**: `autolock::decide` is now a pure function, and its five tests cover the 300 s
  boundary from both sides, the wall-clock jump, sleep outranking timeout, a loaded machine not
  being mistaken for a sleeping one, and `0` meaning never. The gap R-09 still carries is unchanged
  and still named: sleep is detected on **wake**, not before it.
- **The IPC transcript**: `target/ipc-session.log`, twenty crossings, in which the master password
  never returns, the stored secret appears in exactly one command, and `copy_field` carries no value
  of any kind.

**The screenshot harness is committed** (`scripts/screenshots.mjs`, `npm run shots`) — next action 2
from the last session, and the third time the "we cannot see the app" gap has cost something. It
serves `dist/` from node, injects a stub `window.__TAURI_INTERNALS__`, and screenshots eight
scenarios in both themes. Two mechanics are worth keeping: the page holds its own `load` event open
with a deliberately slow image, because Firefox screenshots at load and a scenario that has to press
⌘K would otherwise be captured before the keystroke; and motion is disabled in injected CSS, because
a screenshot of a transition differs between runs and that is the difference between a picture and a
regression test. The PNGs are **not** committed — binary, derivative, regenerable in one command,
and a PNG diff tells a reviewer nothing.

CI gained one step: `invoke(` outside `src/lib/ipc.ts` now fails the build. `ipc_session.rs` proves
what the host sends; that grep is the only thing that keeps the set of callers small enough for the
claim to mean anything.

Verified: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings` silent,
all 10 test suites green (23 unit + 10 audit + 2 session + 1 clipboard + 68 core, plus the ignored
manager probe run deliberately), `svelte-check` 0 errors / 0 warnings over 314 files,
`prettier --check` clean, `vite build` green, and sixteen screens rendered. **CI reproduced all of
it**: run 30997297701 on `feature/phase-2-shell-unlock`, nine jobs green, the first time anything
in Phase 2 has built on a runner. One thing it caught that local checks did not — a test file
written after the session's `cargo fmt` run and therefore never formatted; the fix is a one-line
commit and the lesson is that the format check belongs *after* the last file is written, not in
the middle.

**Not verified:** the functional gate line, which needs the packaged app driven by hand; and the
two clipboard numbers, which CI cannot reproduce — `clipboard_clear.rs` prints SKIPPED without a
display and `clipboard_manager.rs` is `#[ignore]`d, so both measurements come from this desktop
and stay that way until someone runs them on another.

### 2026-08-05 (later — the import question, and a merge nobody wrote down)

A short session with one thing it could not do and one it could.

**What it could not do is still next action 1.** G-B′'s functional line needs a human at the
packaged app — create a vault through onboarding, quit, relaunch, unlock — and no automation on
this machine substitutes for it. `ipc_session.rs` already scripts that exact sequence through the
host and passes; what it cannot witness is the app.

**Reconciling the repo against this document turned up something the document did not know:**
Phase 2's branch was merged into `development` on 2026-08-05 at 17:47, twelve minutes after the
last save here, as PR #1 — with an **empty body** and the gate at 4 of 5. The Git Workflow
Standard asks a phase PR to carry the gate evidence precisely because the commits are already the
changelog; an empty body puts nothing in the place a reviewer looks. Nothing is lost, because the
evidence is in the phase document and CI was green on the branch first. It is written into
*Current phase* rather than fixed silently, since the durable risk is not the missing prose — it
is that a merged branch reads, months later, as a passed gate.

**R-29 is closed as D-42: Bitwarden's unencrypted JSON export, and nothing else.** It was chosen
for work because Phase 3's entry check names it as an external dependency that must be resolved
*before* the first task, and — unlike everything else on the next-actions list — it does not
depend on the gate.

The survey went four sources deep and stopped when the answer stopped moving. The format choice
turned on a constraint that belongs to this project rather than to the formats: N-02 puts every
plaintext byte inside `trustvault-core`, so an importer parses **externally supplied input inside
a Tier-1 crate** under `forbid(unsafe_code)`, no-panic lints and a 90 % coverage floor. Bitwarden's
JSON needs `serde_json` and carries TOTP seeds, folders, custom fields and typed items. KDBX would
have needed AES, Twofish, ChaCha20, Argon2 and an XML parser in that same crate — `keepass-rs` is
maintained and capable, and that was still the wrong trade. CSV, which has the widest reach, has
no schema at all.

**The transferable finding was not about formats.** Every importer surveyed fails the same way:
a field disappears without a word — folders ignored, TOTP seeds landing in a note or nowhere,
attachments gone. So R-29's acceptance criterion is not "it imports" but *every field is either
mapped or named in a refusal*, and the import is one transaction with a preview before it lands.
That is now the wording in `trustvault-requirements.md` and the first line of Phase 3's import
tasks.

Written down in the documents that already existed, not in a research note: R-29's row and the
open-questions list in `trustvault-requirements.md`; five out-of-scope rows in
`trustvault-project.md` (CSV, KDBX, `.1pux`, encrypted exports — and **export from TrustVault**,
which no requirement ever asked for, so recording it is noting an absence rather than deciding
something new); D-42 here; six tasks, a deliverable, a gate line and an entry-check line in
`phases/phase-3-surfaces.md`, taking it from 25 tasks to 31.

One thing D-42 deliberately does not settle, and it is flagged in next action 3: `Item` has no
home for a **custom field or a folder** today, and a Bitwarden export has both. The entry check
owes that answer before the first import task.

### 2026-08-05 (G-B′ passes, Phase 3 opens)

**The author ran the functional line and it worked.** Vault created through onboarding against a
real file, window quit, app relaunched — and the relaunch landed on the **lock screen**, not
onboarding, which is D-40 doing in the app what `ipc_session.rs` proved through the host — then
unlocked with the password. **G-B′ is passed and Phase 2 is closed at 26 of 26 tasks and 5 of 5
gate lines.**

Worth naming what kind of evidence that is, because it is the only line of its kind in the project
so far: an **observation**, not a measurement, and nothing reproduces it. Every other line of G-B′
is a number a CI runner re-derives on every push. This one exists because no harness can watch the
app be an app.

**Phase 3's entry check passed, all six boxes**, and the branch was cut afterwards —
`feature/phase-3-surfaces` from `development`, in that order, because a branch cut first is a phase
opened by habit rather than by check. Four findings changed the phase rather than merely being
recorded:

- **The gate's first line was half met before the phase started.** D-36 drew all 15 surfaces
  disabled, so "all 15 surfaces exist and are reachable" reads as nearly done and means nearly
  nothing. Re-worded to **reachable meaning wired to a command**, with the phase's last task a
  sweep for anything still disabled or wired to a stub. This is the risk D-36 named on the day it
  was taken, arriving on schedule.
- **The 7-day daily-drive clock moved to the front.** It cannot start until the author's own
  passwords are in a vault, which needs `add_item` **and** the importer. Left where the kickoff
  put it — at the end, after everything else — it is not a test of anything, it is a week of
  waiting. Import and item creation are now the first two task groups.
- **Four tasks were added ahead of the code**, all of them things this project has already learned
  the price of: extend `docs/ipc-contract.md` before writing the commands (falsified within the
  hour twice now), settle the item model for custom fields and folders (D-42's unsettled
  consequence), decide `generate_password` as a decision rather than a patch, and start
  `docs/keyboard-audit.md` as a checklist to build against instead of a form filled in at the end.
- **Two glyphs became a task rather than a note.** `docs/icon-gaps.md` has recorded since Phase 2
  that `ssh_key` and `api_key` both fall back to `terminal`; this is the phase where all seven
  types become creatable, so that is the day two of them stop being distinguishable in the list.
  D-28's "no upstream" cost, arriving as a line item exactly as it was warned it would.

Phase 3 is **0 of 39** — and the denominator itself was a finding: the kickoff document claimed 25
tasks and listed 26, the second phase document in a row off by one. Counted rather than carried
forward. Then 6 with D-42 and 7 at the entry check.

Nothing was committed this session and nothing needed to be — the whole document set is
deliberately gitignored (`.gitignore` lines 15–30: the process record stays on this disk while the
code is public), and no code changed. Which is its own reminder: **these files have no remote
copy.** They live and die with this machine unless they are backed up somewhere else.

### 2026-08-05 (Phase 3, first working session)

The "before any command is written" group, closed in one sitting: 4 of 39, and **no command
implemented**. That is the shape the phase document asked for and it is worth stating plainly,
because four ticked boxes with nothing running looks like a slow session and is the opposite.

**`docs/ipc-contract.md` was falsified within the hour, for the third phase in a row.** Fifteen
Phase 3 commands went into it — mutation, search, tags, TOTP, multi-vault, import, the generator —
and `tests/ipc_audit.rs` check 1 failed the moment they were written, because it compares the
registered set against the documented set in *both* directions and none of the fifteen exists yet.
The fix went the stronger way rather than the looser one: a declaration may carry `// planned`, and
the harness now asserts a planned command is **not** registered as well as that a shipped one is.
So the marker cannot park a command that quietly shipped, and implementing one means deleting its
marker in the same commit. The document can now run ahead of the code without the check going
soft, which is the only way "contract first" survives contact with a harness that enforces it.

Four decisions, three of which were not on the task list:

- **D-43**, the one that was — custom fields and folders. One stored bit on `Field`, no folder key
  at all. The argument that settled the first half is that inferring customness from a label
  schema is the mistake §6.3 already refuses to make for `secret`, one step along, and it fails in
  the direction that costs something: an imported custom field called "Username" merging into the
  login's own and pushing a real credential into `history`, where no v1 surface can reach it. The
  two tests that matter pin exactly that and the duplicate-label case, not the happy path.
- **D-44** — `generate_password` becomes the fourth sanctioned command. The clipboard-only
  alternative the contract itself suggested is genuinely better on paper and loses to the surface
  being built: the design shows the password with a regenerate button, because the user is
  deciding whether to accept *this* one. What buys the crossing back is that generation moves onto
  `getrandom`, where R-06 and D-23's CI grep already live, instead of `crypto.getRandomValues` in
  a webview — two generators, one of them "the real one", is a distinction that lasts as long as
  the person who remembers it.
- **D-45** — a TOTP code is not a `Secret`. The contract's §10 had left this open *with the
  warning attached* ("it expires soon" is the argument that ends with secrets in lists), so the
  decision deliberately rests on three properties a password lacks rather than on the lifetime,
  and it is bounded in writing: one item at a time, never batched, never in a list.
- **D-46** — palette search matches in Rust. R-16 wants usernames and URLs, which §6.1 *permits*
  to cross; that is what made it worth a row rather than a shrug. Permitted and still the wrong
  trade: the whole identifying surface of the vault into an unwipeable heap on every render, to
  save one round trip against a budget with room for it.

**`docs/keyboard-audit.md` exists and every box in it is unticked** — 7 global rules, 17 surface
rows. It found something on the day it was written, which is the argument for writing it first: the
gate line says 15 surfaces and the source has 17 rows' worth. The rows are enumerated from
`src/lib/`, and reconciling the two is owed at the gate rather than resolved by picking a number
now.

One near-miss worth recording because it would have broken CI rather than a test. The strengthened
sanctioned-set check was first written to assert that the **decision log** carries D-44 — via
`include_str!("../../trustvault-state.md")`. That file is gitignored (line 29): it compiles here
and would not compile in a fresh clone, so a public CI run would have failed on a missing file with
no obvious cause. Caught by reading `.gitignore` before committing. The check now asserts the
contract cites the decision by number, which is the most a tracked file can say about an untracked
one, and the limitation is written into the test's own doc comment.

Green locally before commit: `cargo fmt --check`, `cargo clippy --workspace --all-targets
-D warnings`, `cargo test --workspace` (all suites, 10 in `ipc_audit`), `svelte-check` (0 errors
over 314 files), `prettier --check`.

### 2026-08-06 (the first three commands)

`add_item`, `update_item` and `delete_item` are implemented, registered, and their `// planned`
markers deleted in the same commit — which is the sequencing note from the last session working as
intended rather than a risk that materialized. Twelve of the fifteen Phase 3 commands remain.

The session started from an uncommitted `FieldEdit` and `Item::apply_edits` in the working tree,
written but untested. What they encode is the rule the contract called the most load-bearing detail
in §6.4, and the type does the work rather than a comment: **two variants, not one struct with two
`Option`s**, so *create a field whose value is unchanged* — which means nothing — cannot be
expressed at all. `EditField::into_edit` is where the wire shape's extra freedom is refused instead
of being given an invented meaning, and it refuses with `internal`, because the only caller is our
own webview and a request in that shape is our bug.

Three ordering decisions inside the commands, none of them visible from outside and each wrong in a
way nothing would report:

- **`apply_edits` validates the whole list before writing anything**, so a refused edit leaves the
  item exactly as it was. That guarantee is only worth having if the title and tags have not
  already been written by then, so `update_item` applies the fields *first* and renames after.
- **Every mutation saves before returning**, so "the command succeeded" and "it is in the vault"
  are one event. The two states are indistinguishable on screen, and the one that loses the item
  the user just typed is the one that looks identical.
- **Omission deletes, and a deletion does not reach `history`** while an overwrite does. The
  asymmetry is deliberate: an overwrite is usually a mistake worth recovering from, a deletion is
  an instruction to stop holding the value.

`tests/ipc_session.rs` no longer seeds through the core. It creates its item through `add_item`,
quits, relaunches, unlocks, reads it back, edits it, and deletes it — the Phase 3 exit gate's first
line **minus the human at the keyboard**, which is the half that stays owed. The transcript now
carries a second secret with a stronger claim on it: the password typed into the *edit* form
crosses inbound, so every response after that point is a chance to echo it, and the assertion is
that it comes back at exactly one place — the reveal that asked for it.

**The harness was checked for vacuity rather than trusted.** `EditField::into_edit` was temporarily
changed to read `value: null` as an empty string — the exact failure §6.4 describes — and the
session test failed on the right assertion, that the untouched field kept its value. Reverted
immediately. A harness whose failure mode has never been observed is a harness nobody knows the
strength of.

**Only one checkbox moved**, from 4/39 to 5/39, and that is the honest count rather than an
oversight. Every task in the item-management group is worded for a *surface* — "Add dialog", "Edit
and delete", "Delete confirmation" — so three commands landing does not tick any of them. Noted in
the phase document at the group heading, because the alternative reading, that the work was not
done, is equally wrong. The wiring is the next session, and it is exactly where D-36's named risk
lives.

Green locally: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`,
`cargo test --workspace` (every suite; 84 in the core, 2 in `ipc_session`), and core coverage at
**99.19 % lines / 97.41 % regions / 100 % functions**, with `model.rs` at 99.80 % — the one missed
line is the unreachable arm in `apply_edits` that the validation pass has already excluded.

### 2026-08-06 (the surfaces, and the boundary that was never connected)

The three commands got their surfaces: the New-item dialog saves, the detail pane's Edit opens a
real dialog, and the delete confirmation deletes. All three go through `src/lib/ipc.ts` and none
through a stub, which is the thing D-36's risk was about. Two of the item-management boxes tick;
the other three do not, and each carries its reason in the phase document rather than being
quietly counted.

**The wiring found that the boundary had never worked.** Reading `strings` on the built binary
while checking how arguments are named turned up `reveal_field…itemIdfieldId`: Tauri v2's
`#[tauri::command]` renames arguments to **camelCase** by default, and `tauri::ipc::CommandItem`
looks the key up exactly with no fallback, so the host wanted `itemId` while `src/lib/ipc.ts` sent
`item_id` — which is what `docs/ipc-contract.md` prints, and what that file converts to on purpose
so the wire format is exactly the document. `get_item`, `reveal_field` and `copy_field` had never
been callable from the webview. D-47.

Three things about it are worth more than the fix:

- **It survived two phases and a gate.** Every command exercised until today happens to have
  single-word arguments — `path`, `password`, `name`, `settings` — so nothing discriminated. The
  first two-word argument to be called in anger would have failed on day one; there was no item to
  call one on until `add_item` landed yesterday.
- **Our own harness was structurally blind to it.** The `_inner` split that lets `ipc_audit.rs`
  drive the *real* command bodies — written precisely so the harness does not test a
  reimplementation — skips the argument decoding, which is the part that was broken. This is the
  counter-example to Phase 2's "harnesses catch drift on their first run": `ipc_session.rs` scripts
  a whole session, passes, and passed all along, while the application did not work. So did the
  screenshot harness, which stubs `invoke` itself.
- **What would have caught it is the exit gate's first line**, run by hand: create an item through
  the UI, quit, relaunch, unlock, read it back. That line is in this phase's gate because D-38 moved
  it out of G-B′ when Phase 2 could not satisfy it. It can be satisfied now, and it is the next
  action.

The fix is `rename_all = "snake_case"` on **every** command rather than the ones that need it
today, asserted by a new check in `ipc_audit.rs` and verified non-vacuous by removing the attribute
from one command and watching it fail. `snake()` in `ipc.ts` was also taught to descend into arrays
in the same commit — `fields: EditField[]` is the first array of objects to cross outbound, and the
omission would have become a silently dropped key the day one of them was named with two words.

**Two more decisions the task list had not anticipated.** **D-48**: the prototype draws no edit
surface at all — its toolbar Edit button has no handler — so this is the first surface in the
project built without pixel values to follow, and it is **field-driven, not type-driven**. That is
forced rather than chosen: an item is a list of fields, `update_item`'s third rule is that omission
deletes, and a form built from `ITEM_FIELDS[kind]` would delete every custom field an import
created, in silence, on a rename. **D-49**: the delete confirmation promised "It goes to Trash for
30 days first", against a Trash view that holds nothing and a command with no such window. It was
false the day it was written and it is the last sentence a user reads before clicking.

The New-item dialog stopped being markup-per-type and became data — `src/lib/shell/itemFields.ts`.
Markup describing an item and code building the payload are two statements of the same thing, and
the first edit touching only one of them saves a field under the wrong label or, worse, a secret
unmasked.

One finding recorded rather than fixed: **a tag can be chosen but never created.** The chips in
both dialogs are the tags already in the vault, so a fresh vault offers none and no first item can
ever be tagged. It belongs to the *Tag assignment and filtering* task, which stays unticked.

`docs/keyboard-audit.md` gains row 18 for the edit dialog, which takes the table to **18 rows
against the gate's 15** — widening a reconciliation that section already owed rather than changing
its shape. Row 18 carries the finding it was written to catch, and that one *is* fixed: pressing
Replace on a secret row swaps a disabled input for an editable one in the same position, and
nothing moves focus there on its own, so the dialog moves it.

Green locally: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`,
`cargo test --workspace` (11 in `ipc_audit`, 2 in `ipc_session`, 84 in the core, every suite),
`svelte-check` (0 errors, 0 warnings), `prettier --check`, `vite build` (133.8 kB JS / 42.5 kB
gzip), and the screenshot harness rendering twenty screens in both themes.


### 2026-08-06 (the importer)

The Bitwarden importer, core-side: `crates/trustvault-core/src/import/` with `bitwarden.rs` under
it, `Vault::preview_bitwarden` and `Vault::import_bitwarden`, and `import_preview` /
`import_commit` on the command layer with their `// planned` markers deleted in the same commit.
One new dependency, `serde_json`, which is the one D-42 budgeted for and the reason that format
was chosen over KDBX.

**The schema was read, not remembered, and that is the whole session in one sentence.**
`CipherType` in `bitwarden/clients` carries **eight** types where every write-up says four: 5 is
an SSH key — which TrustVault has a type for, so a parser written from memory would have dropped
it silently — and 6, 7 and 8 are a bank account, a driving licence and a passport. Two decisions
came out of that rather than out of the task list. **D-50**: a type we do not have becomes a
secure note carrying its own keys as custom fields, through one generic path, so the ninth type
imports too. **D-51**: an unknown *key* is a refusal, so a schema that keeps growing cannot grow
past us in silence — which is the mechanism behind the failure D-42's survey found in every
importer it looked at.

The finding that cannot be fixed here: **an export can only ever produce five of our seven
kinds.** Bitwarden has no API-key type and no Wi-Fi type, so the exit gate's own R-29 line — "a
Bitwarden export covering all seven item types" — cannot be met as written. Reaching those two
would mean guessing from a title, which is the inference D-43 exists to prevent, so there is now a
test asserting an import never produces either kind. The gate line is left unticked and unedited,
with the proposed re-wording in the open questions: a gate's vocabulary is the author's to change.

R-29's acceptance criterion is tested as a **rule**, not as a list.
`nothing_in_the_export_is_dropped_in_silence` walks every leaf of the fixture and demands that its
value is either in the vault or named in a refusal. Verified non-vacuous the way the session
harness was — one mapping deleted on purpose — and it fails naming `items2.card.brand = Visa`.
Sixteen named tests pin the individual mappings around it, including the one that matters most to
D-42: the TOTP seed is in `2FA secret` **and** is not in the note.

Two smaller things, both of the same shape as bugs this project has already paid for. Every
value-bearing string in the parse structures deserializes straight into `SecretString`, so the
foreign vault's plaintext is zeroized when the parse drops rather than left in a heap nothing
wipes. And `ipc_audit.rs`'s `rename_all` check — the one D-47 added — iterated a hand-written
list of command modules, which is the same shape as the bug it exists for: it now reads
`src/commands/` and fails if a module is missing from the list, which is how `import.rs` was
covered on the day it was written.

What is **not** done, and is not a wiring job: the import surface. D-36 drew the design's fifteen
surfaces, the design predates D-42, so there is no import anywhere to wire. It needs a place
(Settings, with `MASTER.md` as the only authority, as in D-48) and a **file picker**, which is a
dependency this app does not carry. `tauri-plugin-dialog` is the candidate and it would also
retire D-36's drawn-and-inert "Open vault file…", so one prior-art survey answers two Phase 3
tasks. Left undecided on purpose: a dependency is chosen once in an hour and paid for over the
project's whole life.

Green locally: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`
(zero warnings), `cargo test --workspace` — 170 passing across every suite, including 17 in the
new `import_bitwarden`, 11 in `ipc_audit` and 97 in the core's unit tests. Core coverage measured
against N-03's 90 % floor: **98.58 % lines / 96.73 % regions**, with the new `import/bitwarden.rs`
at 96.45 % lines. The frontend was not touched this session.

Both sessions' work went onto one short branch, `feature/wiring-and-import`, as two commits —
the surfaces first, the importer second — and merged into `feature/phase-3-surfaces` with
`--no-ff`. Splitting them was not cosmetic: two files carried changes from both sessions —
`src-tauri/tests/ipc_audit.rs` and `docs/ipc-contract.md` — so the first commit holds the
*wiring-only* form of
each, with `import_preview`/`import_commit` still marked `// planned` there. The first commit was
checked out into a throwaway worktree and its full suite run there, so "each commit builds" is
measured rather than assumed — a split that leaves the first half broken is a split that makes
`git bisect` lie.

### 2026-08-06 (the generator)

The generator group closed whole — four boxes, host and surface in one sitting, Phase 3 at 15 of
39. `generate_password` is the **fourth sanctioned command** the contract has named since D-44 and
`copy_generated` is beside it, both markers deleted in the commit that implemented them. It is the
first time D-37's two closed copy paths open, and the first D-36 control to become a real command
rather than a stub.

Three things came out of building it that are worth more than the four ticks:

- **The webview generator is deleted, not kept beside the new one.** `src/lib/passwords.ts` was
  good code — `crypto.getRandomValues` with rejection sampling, deliberately never the prototype's
  `Math.random` — which is exactly why deleting it was the decision rather than the cleanup. D-44's
  argument was already written down: two generators with one of them being "the real one" is a
  distinction that survives as long as the person who remembers it. All three surfaces that mint a
  password now call the one command.
- **The prototype's footer promised something v1 does not ship.** *Copy & autofill* — and autofill
  is a browser extension `trustvault-project.md` puts out of scope. Second false promise found in a
  drawn-but-inert control in three days, after D-49's Trash copy, and the pattern is now visible:
  a control that was never wired was also never read. Worth a sweep of the remaining disabled
  controls' labels **before** wiring each, not after. The ambiguity toggle went the other way and
  for the same reason — the prototype's own digit chip says *2–9*, so the exclusion is a property
  of the sets, not an option. Both in D-52.
- **Nothing the user does in the webview resets the auto-lock clock.** Found by having to decide
  whether `copy_generated` should `touch()`. Only vault-class commands do, through `with_vault`, so
  a user filling in the New-item dialog for longer than `auto_lock_seconds` is locked out mid-form
  and loses what they typed — `add_item` answers `locked`. Not fixed here: the fix is a decision
  (an ambient heartbeat command, or activity tracked in the host window) and the tempting wrong
  one is to lengthen the timeout. It is the first finding the 7-day drive would have produced,
  arriving a week early.

The core generator is 308 regions at 98.05 %, with the whole crate at **96.77 % regions / 98.68 %
functions** against N-03's 90 % floor. Its randomness goes through `aead::random`, so the two CI
greps that confine `getrandom` to one file and forbid any seedable RNG in the crate cover it with
no new rule. Rejection sampling rather than modulo, every selected set guaranteed to appear, and
the guaranteed characters Fisher-Yates shuffled afterwards — the last of those is the one a
reviewer would not miss but a rewrite would: without it, position 0 is always lowercase in every
password the app has ever produced.

Two things were fixed that this work did not cause. The screenshot harness had to be taught the
new commands with a **fixed** password, because a shot that differs on every run cannot become the
baseline the CI question needs. And `npm run fmt:check` was **already red on the branch** — the
Bitwarden fixture committed earlier the same day carries one line over width. Reformatted rather
than ignored: a prettified export is closer to what Bitwarden actually emits, not further from it.

Green locally: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`
(zero warnings), `cargo test --workspace` — 12 in `ipc_audit`, including the new
`the_generator_returns_one_secret_and_needs_no_vault`, which pins both halves of the command's
class: exactly one secret per invocation (R-10) and no vault required, the second being why it is
absent from the locked-state check. Frontend: `svelte-check` 0 errors over 315 files,
`prettier --check` clean, `vite build` at 134.78 kB JS / 42.55 kB gzip. The twenty screens were
re-rendered and the generator read by eye in both themes.

### 2026-08-06 (the palette)

The command palette's matching moved into the core, which is the fifth Phase 3 command and the
first one D-46 had already decided before the code existed. What the surface had been doing until
this session is worth stating plainly rather than filing under "wired": it filtered a **client-side
copy** of the item list on `title` and `tags` — two of R-16's four haystacks, and the exact shape
D-46 rejected. It looked finished, because a palette that finds `GitHub` when you type `git` is
indistinguishable from one that searches what the requirement says.

Four of the group's five boxes tick, and only one of them is new work. Three are the surface D-36
drew, read against R-16 line by line and found to do what it says — ⌘K/Esc/arrows, actions beside
items, Enter copies and ⇧Enter opens. That is **not the same as verified by hand**: row 11 of
`docs/keyboard-audit.md` is still unticked, and S-08 is what ticks it.

**The rule about what is searched was falsified within the hour — the fourth time in four phases,
and by our own harness this time.** It was written as R-16's own words (non-secret fields of kind
`username`, `url`, `email`) and the IPC audit fixture failed immediately: `Item::set_field` guesses
`kind` from `secret`, so the fixture's username is a `text` and was not searchable. The two
attributes are not interchangeable — `kind` is how a field renders and is only as accurate as
whoever created it, `secret` is what the user declared — and only one of them is a boundary. D-53
makes `secret` the only gate, which searches more than the requirement asks for and cannot be
quietly wrong; what stops a note body outranking a title is now the weights (title 300, tag 200,
field 100) rather than the haystack list, and there is a test named for that.

The security half of the same rule got the test it deserved rather than a comment: **a query equal
to a stored password returns nothing.** A palette that ranked on secret values would confirm a
guessed password through the order of its rows — nothing revealed, nothing crossing IPC, no audit
entry, and the answer on screen. Pinned in the core for every `FieldKind` and in `ipc_audit.rs`
against the planted secret.

**The reference vault did not exist.** S-02, S-04, S-07 and R-11 have all named
"`trustvault-core`'s `benchfixture` helper, 1 000 items, 4 fields each" since kickoff and nothing
implemented it — a requirement that reads as satisfied because it names a file. Written now behind
a feature so it is absent from the shipped library (a fixture generator compiled into the app is a
way to put a thousand fake items in a real vault), and derived from the item index with **no random
number generator at all**: R-06's CI grep forbids a seedable one in this crate, and hand-rolling one
to slip past the grep would be the rule broken with the evidence removed. The self dev-dependency
that turns the feature on for benches is invisible to the N-02 boundary job, which reads
`--edges normal`.

S-04's host half measures **0.85 ms p95** (median 0.44, max 1.65; 34 000 samples over six queries
and every prefix of each). That is 1.7 % of the 50 ms budget, so the criterion now rests on the IPC
hop and the render rather than on the matching — and the gate line stays **unticked**, because
nobody has taken the end-to-end measurement. The instrument is in place: the palette emits
`palette-keystroke-to-render`, which is the "performance marks" S-04 names as its method. Taking it
needs the reference vault *in the app*, so the fixture now has to become a `.tvault` on disk.

Two smaller things. A tag can be **created**, not only chosen — the chips were the tags already in
the vault, so a fresh vault offered none and no first item could ever be tagged; both dialogs now
draw the Tags control unconditionally, where before it was hidden exactly when it was needed. And
`list_tags` stays `// planned` on purpose: `ItemSummary.tags` already carries every tag to the
frontend under §6.1, so the sidebar and both dialogs compute their lists *and counts* without it,
and shipping it would add a second path to data the webview legitimately holds. Raised as an open
question rather than deleted from the contract.

Green locally: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`
(zero warnings), `cargo test --workspace` — 16 test binaries green, `ipc_audit` now at 13 with
`a_search_response_carries_no_values_and_secrets_are_not_searchable`. Frontend: `svelte-check`
0 errors over 315 files, `prettier --check` clean, `vite build` at 134.95 kB JS / 42.64 kB gzip.
The twenty screens were re-rendered; the palette, New item and Edit item read by eye in both themes.
One thing that was not the code: `target/` held a truncated test binary and a corrupt incremental
cache from an interrupted build, which presented as `Exec format error` and then as an internal
compiler error. Deleted, not diagnosed further.

### 2026-08-06 (TOTP)

The TOTP group closed whole — core, both commands, both surfaces, three boxes. Phase 3 is at 23 of
39. Two things about it are unlike the four groups before it, and both are worth keeping.

**The specification came with its own numbers, and the vectors passed on the first run.** RFC 6238's
Appendix B publishes 18 test vectors — six times × three algorithms — and every one of them passed
the first time `cargo test` ran. That has not happened before in this project, and the reason is not
that the code was written more carefully: it is that for once the thing being implemented was
arithmetic somebody else had already numbered. Verified non-vacuous the way everything else here is,
by breaking it on purpose: dropping the sign mask from the dynamic truncation fails the vector test
and three others with it.

That is also the whole of **D-54**. The dependency is `hmac` + `sha1` + `sha2` from RustCrypto and
*not* `totp-rs`, which is the obvious choice and wraps exactly those three: what it drags with it is
a second base32 beside `data-encoding`, a second constant-time compare beside `subtle`, and
`otpauth://` parsing behind a feature pulling `url` **and** `urlencoding` — two crates to split a
query string whose four keys we control. The 0.13/0.11 line was taken over the older 0.12/0.10 one
even though the older would have reused the `digest 0.10` argon2 already pulls through blake2,
because only the new line has `zeroize` wiring and the HMAC state holds the seed. The cost is
written down rather than absorbed: two `digest` versions in a Tier-1 crate's tree until argon2 0.6.

**The risk was the parser, and this time it was found by reading our own code first.** Four phases
running, the thing that would have gone wrong was caught by a harness after the fact — the contract
in Phase 1 and Phase 2, `ipc_session.rs` and D-40, D-47's argument casing, D-53's `kind` gate. This
one was visible before a line was written: `import/bitwarden.rs` has stored **either** a bare base32
seed **or** a whole `otpauth://` URI since the importer landed two sessions ago, because that is what
Bitwarden emits. A generator reading only the first would have computed codes from *the letters of a
URL* for every imported item and reported success doing it. `otpauth://hotp/…` is refused rather
than read as TOTP for the same reason one level up — HOTP counts logins, not seconds — and the URI's
`digits`, `period` and `algorithm` are honoured because ignoring them produces a perfectly formed
code that is wrong every single time. Each is a named test.

Three decisions the task list had not anticipated, all small and all written down rather than
absorbed:

- **`totp_code` is the first vault-class command deliberately left un-audited.** The pane refreshes
  it every step, so an entry per call writes 120 an hour with the pane open and evicts every genuine
  reveal from D-31's 1000-entry cap before lunch. An audit log that is mostly its own noise is worse
  than none, because it still looks complete. The *seed* being revealed still goes through
  `reveal_field`, which records it.
- **The code's copy path already existed.** It goes through `copy_generated`, the third caller of a
  command whose name says "generated" and whose safety argument covers this one unchanged: the value
  is one the window already holds legitimately (D-45), the host reads nothing, and what the call
  buys is the clear scheduled in Rust. The alternative — a `copy_totp` taking an item id and
  re-deriving the code host-side — is one more command and one more path to a stored seed, for a
  string the caller is already displaying. Written into §5 rather than left as a convention.
- **D-55**: the prototype's *Scan QR on screen* button is not drawn. It needs screen capture, which
  v1 does not ask for anywhere and which is a permission this app is better off never holding. Third
  prototype promise removed in three days, after D-49's Trash window and D-52's autofill — which is
  a pattern rather than three incidents, and the pattern is that the design was drawn before the
  scope table existed.

**The preview went into the Edit dialog as well, which the task did not ask for.** A seed can be
replaced there, so a validator guarding one of the two ways in is one that gets reported as
"sometimes it checks" — and the edited seed is the more dangerous of the two, because the item
worked before the edit. That is what turned it into `TotpPreview.svelte` instead of forty lines
inside the Add dialog.

Two findings from the harnesses. **`ipc_audit.rs` failed on its first run again**, because
`commands/totp.rs` was not in the module list its directory check compares against — second time
that check has earned itself, and this time by construction rather than by luck. And the frontend's
`ErrorKind` union was **missing `not_importable`**, found while adding `malformed_totp_secret`
beside it: the host has returned that kind since the importer landed and `asIpcError` would have
narrowed it to `internal`. It had never been reachable, because import has no surface — which is
exactly the drift contract check 1 cannot see, since both sides "have" the command.

The seed now lives in the audit harness's **shared** fixture rather than in the TOTP test alone, so
checks 2 and 4 assert against a vault that holds one, and contract check 7 is automated in both
halves for the first time. The code half is pinned on the absence of a `code` key in any list rather
than on the six digits, because six digits occur inside a UUID by chance often enough to make a
flaky check that somebody eventually deletes.

Green locally: `cargo fmt --all`, `cargo clippy --workspace --all-targets -D warnings` (zero
warnings), `cargo test --workspace` — every binary green, `ipc_audit` now at 14 with
`totp_returns_a_code_and_never_the_seed`, core at 132 with 12 new TOTP tests. Frontend:
`svelte-check` 0 errors over 317 files, `prettier --check` clean, `vite build` at 140.09 kB JS /
44.19 kB gzip. The screenshot harness is at **twenty-two** screens: it gained a `newItemTotp`
scenario and a `fill()` driver, because a surface that only appears once a field has something in it
cannot be photographed by clicking alone. Both new surfaces read by eye in both themes — the detail
pane's row against the prototype's own geometry, and the Add dialog's preview strip.

One departure from the prototype that is not a decision, only a correction: it drives the ring with
`animation: totp 30s linear infinite`, a free-running CSS loop that starts whenever the element
mounts. Ours is driven by `expires_at` from the host. The appearance is identical and the loop is
wrong in the way that matters — the user is looking at their phone at the same moment, and a ring
that says nine next to a phone that says twenty-two is a ring nobody trusts again.

### 2026-08-06 (settings and vaults)

The largest group in the phase, and the one that leaves **one `// planned` marker** in the whole
contract. Six of the Settings & vaults boxes tick, plus the item-management delete box that had
been half done since the morning — Phase 3 goes 23 → 30 of 39. R-21, R-22, R-27 and the second
half of R-18 land together, because they are one group in the plan and turned out to be one group
in the code as well: the vault commands need `known_vaults`, `known_vaults` shares the settings
file, and the settings file is what the five new fields go into.

**`launch_at_login` is the only setting in this application that writes outside the process**, and
that is the whole shape of the work. Three implementations behind one boolean — an XDG desktop
entry, a `LaunchAgent` plist, a `reg.exe` value — hand-written rather than taken from
`tauri-plugin-autostart` or `auto-launch` (**D-56**; the closest call was `auto-launch`, rejected
because its macOS path adds a login item through AppleScript, which means `osascript` as a
subprocess from a process holding decrypted secrets). What the contract specified in advance and
what the code had to honour is the failure mode: the OS write happens **before** anything is
stored, so a platform that refuses leaves the stored value alone and the toggle snaps back rather
than promising something nothing registered. It is also reconciled against the OS at start-up — a
user who removed the entry through their desktop's own startup tool has said something the settings
screen must not go on contradicting.

**The document falsified this time was `MASTER.md`, not the contract.** Four groups running, the
thing written first and disproved within the hour has been `docs/ipc-contract.md`; here it was the
design system. §3 has said since kickoff that UI Scale "scales the whole `rem` root", and
`tokens.css` has **no `rem` in it at all** — every size is `calc(<px> * var(--ui-scale))` with
`data-ui-scale` on the root. The outcome is identical *only because nothing in the codebase uses
`rem`*, which is a condition rather than a fact. So **D-57** records the divergence, both documents
are corrected in place with the disagreement stated rather than re-worded to look right, and a CI
grep fails the build on any `rem` under `src/`. The failure it prevents is the nasty kind: a
setting that scales most of a screen reads as a layout bug, not as a broken setting.

**Window geometry joins `last_vault_path` as host-owned, and its trap is the sharper of the two.**
D-40's field needs a frontend that does not know about it; this one needs nothing at all. The
webview holds a `Settings` from when its screen opened, so resize the window, then change any
setting, and a frontend faithfully echoing every field it knows about sends back the dimensions
from before the resize. `merge_incoming` keeps four fields now, with a test each. Two of the
geometry rules are choices rather than mechanics and both are written down: a maximized window's
own size is **not** recorded (its dimensions are the screen's, and restoring to them makes
un-maximizing land on a window the size of the display), and the **position is not restored at
all** (a remembered position on a display that is no longer attached opens the window off-screen,
which is indistinguishable from the app failing to launch and cannot be undone from inside it).

**D-47 arrived from the other direction.** `delete_vault`'s confirmation — the check R-18 asks for,
and the single check in this product whose *success* is irreversible — was written into the
`#[tauri::command]` wrapper, which is where it naturally goes and which is exactly the layer the
`_inner` split skips. It would have been the one check no test had ever read. The whole sequence
moved into `delete_vault_inner`, and that is also what makes its three orderings testable: the name
before the lock (a typo must not cost an open session), the key zeroized before the bytes, the file
before the bookkeeping (a failed remove leaves an entry rather than hiding a vault that still
exists). Verified non-vacuous by loosening the comparison to case-insensitive on purpose, which
fails it. `display_name_for` serves both `list_vaults` and the check, which is correctness and not
tidiness: two implementations that drifted would make a vault undeletable through its own dialog,
with the user typing exactly what is on their screen and being told it does not match.

**`known_vaults` grows in exactly one place.** `Inner::opened` replaced the three identical blocks
in `create_vault_inner`, `unlock_inner` and `unlock_recovery_kit_inner`. Consolidating them was not
tidying: the list has to be updated on every path that leaves a vault open, and the fourth such
path is added by someone who copies the vault and the path and does not know there were bookkeeping
lines to copy — after which the switcher is missing the vault on screen. `persist` was changed to
take the state rather than a `Settings` for the same reason one level up.

**The keyboard audit was used as a checklist to build against for the first time**, which is what
its own preamble said it was for. Row 15 asks for ↑/↓ between vaults *including the open one*, and
the open vault's row was drawn `disabled` first — the obvious way to say "you are already here",
and wrong for one reason: a disabled button cannot take focus, so the arrow stops dead on the row
the user is standing in and reads as the key having failed. It is `aria-disabled` instead:
focusable, announced as unavailable, a no-op when activated.

Two pieces of copy. The `io` message said "could not read or write **the vault file**", which for a
failed login-entry write is D-49's mistake one screen over — a sentence read at the moment it
matters, describing something that did not happen; it now names *a* file rather than *the* file.
And `confirmation_mismatch` was in the contract's §4 and **missing from the frontend's `ErrorKind`
union** — the third instance of that exact drift, each time because nothing had yet been able to
reach the kind, and each time invisible to contract check 1 because both sides "have" it.

Green locally: `cargo fmt --all`, `cargo clippy --all-targets` (zero warnings), `cargo test` —
every binary green, host unit tests at 44 with seven new vault-command tests and the autostart
round-trip, `ipc_audit` at 14 with all four new commands registered and their markers deleted in the
same commit. Frontend: `svelte-check` 0 errors over 317 files, `vite build` at 143.66 kB JS /
45.18 kB gzip.

One rough edge named rather than left to be found: `switch_vault` points at a path without checking
the file is still there, so switching to a vault deleted outside TrustVault lands on a lock screen
that cannot unlock. Recoverable through the switcher's Leave. The tempting `is_file` check is wrong
for `restore_last_vault`'s reason — it would silently refuse a vault on a drive that is merely
unplugged — so the real fix is the switcher knowing which files exist, which is a second command
and a decision. Next action 17.

### 2026-08-06 (empty states, and the seventh glyph)

The two boxes nobody had picked up, and they closed the item-management group whole — 7 of 7, the
first Phase 3 group to get there. Both looked like leftovers, and neither turned out to be about a
missing capability: one was about what the application **says**, the other about what it **shows**.

**Every list already had an empty state. Two of them were lying.** `ItemList.svelte` drew one
message across five panes, so in Favorites, in a vault holding fifty items, it read *"No items in
this vault yet"* — and in Trash it read *"Deleted items sit here for 30 days"*, which is **the
sentence D-49 removed from the delete dialog that same morning**, sitting one pane over the whole
time. That is the finding worth more than the fix: D-49 corrected the copy it was reported against
and nobody swept for its siblings, so a false sentence found in one place has to be treated as a
class in the same session, not as an incident. The state is per view now and each branch names what
is actually empty; the action follows the same rule — *Add item* where adding fills the pane,
*Show all items* where it cannot, because Trash and an unused tag are dead ends you leave rather
than fill and R-19 asks for an action, not for a plausible one.

Swept in order: the item list's five views, the palette's results, the vault switcher, Watchtower,
the detail pane, and the sidebar's tag section. Two of those changed. The palette's "no results"
was a line of grey text, which makes a failed search and a failed *load* look identical; it is an
`EmptyState` with a *New item* action, and that action is not the duplicate it looks like — a query
matching no item has also filtered the New item **command row** out of the list, so it is the only
way left to act on what was just typed. The vault switcher's was a bare `<p>`. The sidebar's tags
were left exactly as they are: the section is **absent** when there are none, and an absent section
is not a blank pane — a heading over an empty box would be worse.

**One glyph, not two** (D-58). The gap note has said since Phase 2 that `ssh_key` and `api_key`
share `terminal` and called it the worst gap in the set, which frames the fix as *draw an ssh key*.
Reading `MASTER.md` §8 instead of the gap note reverses it: §8 **assigns `terminal` to the ssh key
by name**. It is that type's glyph and never was a substitute. What §8 does is enumerate six of the
seven types and omit `api_key` — a type with nothing assigned to it, which is exactly how two of
them came to share one for two phases. So `code` (`< / >`) is drawn for `api_key`, in the shipped
set's geometry, and the alternative is named rather than assumed away: a second key-shaped glyph
would have left the real gap open and put three key silhouettes in one list, told apart by the bow,
which is not what anyone scanning a list looks at.

§8 was corrected in place twice — for the new assignment, and because it **still said Lucide**,
which D-28 replaced two phases ago. Nobody had gone back for it, in the same section as the gap it
caused. That is D-57's practice applied one section up rather than a new one.

**The harness was made to shoot what nobody visits.** Three scenarios in both themes — an empty
vault, Trash, and the palette with a query matching nothing — because Trash is empty **by
construction** in v1, so its copy lives on a screen the author will never open again, which is
precisely where a false sentence sat undisturbed for a day. It cost one change to
`scripts/screenshots.mjs` itself: a scenario may now replace the item fixture outright, since "the
list is empty" is a different fixture and not a flag on the same one. All twenty-five screens were
re-shot, which is also how the two type glyphs were confirmed distinguishable at 16px rather than
asserted to be.

Green locally: `svelte-check` 0 errors over 317 files, `prettier --check` clean across the repo,
`vite build` at 144.34 kB JS / 45.35 kB gzip. No Rust changed in this session.

### 2026-08-06 (the import surface, and the first plugin)

**One box, and it is the box the phase's critical path ran through.** Six tasks remain and every
one of them now wants the app running — the import surface was the last thing in Phase 3 that could
be built from a text editor. What stood between here and the 7-day daily-drive clock was never a
screen: it was that TrustVault had **no way to let a user choose a file**, which is a dependency
decision, and the state document has said so since the importer landed.

**D-59 is that decision, and its shape is "adopt it, grant the webview none of it".** Two Phase 3
tasks wanted a picker and one prior-art survey answered both — R-29's import, and the switcher's
*Open vault file…*, drawn-and-inert since D-36 for exactly the reason the manifest still states.
`tauri-plugin-dialog` 2.7.2 is now the only plugin in the application, registered in Rust so that
`commands::picker`'s two argument-less commands can open a native dialog, and
`capabilities/default.json` is still `core:default` alone — the plugin's own `open`, `save` and
`message` are **denied**, and `pick_import_file` / `pick_vault_file` are the only doors.

**The option that had to be refused on principle was the one every web-shaped app reaches for
first.** `<input type="file">` costs no dependency at all and is architecturally wrong rather than
merely more work: it hands the **webview** the file's bytes, and for an import those bytes are
another password manager's plaintext in the one heap `CLAUDE.md` says can never be wiped. So the
picker returns a **path** and the host opens the file — the same rule §2 already states about vault
data, arriving from the opposite direction.

Between the two host-side candidates it was close. Bare `rfd` 0.16 is one crate fewer and the same
dialogs, since the plugin wraps it. What the wrapper contains is the part that is not code we would
enjoy owning: a **synchronous** Tauri command runs on the main thread, where `blocking_pick_file`
deadlocks, and macOS falls back to a sync dialog without an `NSApplication` — so we would re-derive
`run_on_main_thread` + `block_on` + parent-window attachment and then maintain it across every
Tauri upgrade, to save a crate already in the tree.

**Reading the crate found two costs no README carries, and both are now tests.** It pulls
`tauri-plugin-fs` in as a library dependency — no fs commands are registered, but the crate is in
the tree and the next `cargo audit` will see it. And its init script **replaces `window.alert` and
`window.confirm`** in our page. The second is the sharp one: upstream's replacement `confirm` is
**async**, so `if (confirm("Delete this?"))` tests a promise and is true always. A confirmation
written the ordinary way would now confirm itself.

**That check failed on its first run**, the third time in this phase a new harness check has earned
itself immediately. `DeleteDialog.svelte` declared `async function confirm()`, shadowing the global
— which was *safe*, and that is the problem: it is safe exactly until somebody moves the call, on
the one dialog in this product whose **success** is irreversible. It is `remove` now. The rule as
written is blunt on purpose — the identifier may not appear in `src/` at all — for the reason the
`rename_all` check is uniform: a rule with an edge is a rule somebody argues past. It cost one
relaxation, comment lines are skipped, because the note explaining the rename has to spell the call
out to be worth reading and a rule that forbids describing itself gets deleted rather than obeyed.

**The surface itself is the second in this project designed without the prototype** (D-48 was the
first), so `MASTER.md` was the only authority again. It lives in a new **Import** group in Settings,
three steps, and the middle one is the requirement: the refusal list is drawn **in full**, with no
"and 12 more". R-29 is the rule that nothing is dropped in silence, and a list that hides its tail
is that rule broken one indirection out, where it looks like restraint. The report on screen after
the import is the **commit's**, not the preview's — §6.8 has `import_commit` re-read the file, so a
file edited in between imports as it is now, and leaving the preview up would report the losing
side of a real race as fact.

**Export is absent rather than disabled**, which is a different call from D-36's and worth the
distinction: D-36's rule covers surfaces whose *command* has not landed. `trustvault-project.md`
puts export out of scope for v1 entirely, so a greyed control would be a promise the product has
decided not to make — the same reasoning as D-52's *Copy & autofill* and D-55's *Scan QR*, which is
now four prototype-or-adjacent promises removed in four days.

**The screenshot harness caught a layout bug before a human saw the screen**, which is the first
time it has done that rather than merely recorded state. `.lede` was a flex container so it could
carry a status glyph, and flex makes every child a flex item — the `<strong>` inside the intro
paragraph became a **column of its own** and the sentence rendered as three fragments side by side.
Three scenarios were added in both themes (intro, preview, done), and taking them needed one change
to the harness: the load-hold is **per scenario** now. The import flow is the first drive with
three clicks in it, the default 900 ms expired mid-sequence, and the result was a photograph of the
previous step — a shot of the wrong state still looks like a shot, which is the worst way for this
tool to fail.

**One empty state was falsified by this change and swept in the same session.** The vault
switcher's carried a comment explaining that it had no action *because this build could not pick a
file*; that stopped being true the moment the picker landed, and nothing prompts you to go back for
a state written around a limitation when the limitation goes. It offers *Open vault file…* now.
This is the lesson from the empty-states pass one day earlier applied on the day it was owed rather
than a day late.

**`docs/keyboard-audit.md` gained row 19**, and it carries the only line in that table that is not
about our own focus handling: the native dialog is the operating system's, so what has to be
checked is the **return** — a dialog that comes back with focus on `body` leaves the user who just
chose a file with nothing selected and no visible reason. It is also the first row the screenshot
harness cannot help with, since a headless browser has no file chooser to open.

Green locally: `cargo test --workspace` all suites passing including the two new audit checks,
`cargo clippy --all-targets` clean, `cargo fmt` clean, `svelte-check` 0 errors over 318 files,
`prettier --check` clean, `vite build` at 151.83 kB JS / 47.22 kB gzip. Thirty-four screens re-shot
in both themes.

**Not done, and deliberately.** The R-29 fixture wording is still the author's call (open questions,
action 13) — re-wording a gate's own vocabulary while implementing against it is not a thing to do
in the same session. And **nothing here has been run against a real Bitwarden export**: every
assertion behind the importer is against a fixture this project wrote, which tests the parser
against our own reading of the schema rather than against what Bitwarden emits. That is action 19,
and it wants the same sitting as actions 8 and 15.

### 2026-08-07 (the D-36 sweep, and the two checklist lines nothing had ever measured)

**Three of Phase 3's last six tasks closed, and the three left are the three that need a person.**
The accessibility group was the whole session: the D-36 sweep, the focus ring, and the contrast
audit. What did not close is the keyboard audit's nineteen surfaces, which want the pointer
physically unplugged.

**The sweep found four controls and every one of them had gone stale the same way** — the thing
its disabled `title` said it was waiting for had arrived, and nobody went back for the sentence.
That is the shape worth carrying forward rather than the four fixes: a reason written on a
disabled control is a claim with an expiry date on it, and nothing in this repo expires it.

Two removed (**D-61**), two wired (**D-60**, **D-62**). The removals are the D-49/D-55 pattern for
the third and fourth time in three days, which makes it a rule: *a control that promises what v1
does not ship is deleted, and the reason stays in the file where the control was.* The recovery
kit has no file to load, because R-07's kit is whatever the user's own print dialog wrote; *Rename
vault* has no requirement, no command and no prototype behind it.

**One of the two wired was hiding a hole, not a blemish.** `vault_status` answers `no_vault`
exactly once in a vault's life, so onboarding was unreachable ever after and **a user with one
vault could never create a second** — R-22 is a `must` and it was reachable only by somebody who
already had another `.tvault`. Wiring it found the trap beneath: `create_vault` writes the file
whole, and the flow opens with the same default name every time, so keeping "Personal Vault" for
both would have written the first vault over with the second. No confirmation, no undo, and no key
in memory to have warned with. Verified non-vacuous the usual way — with the new check disabled,
the test fails on the overwritten bytes.

**The two §10 lines that had never been measured were never measured for a structural reason.**
The app needs a Tauri host to render anything, so since Phase 0 every claim about how it *looks*
has been a claim about its CSS. `scripts/a11y.mjs` walks the same fake application the screenshot
harness photographs — the stub host moved into `scripts/harness.mjs` so that a shot and an audit
are evidence about one product rather than two.

**895 contrast findings on the first run**, and the token behind them is the one `MASTER.md` calls
"text disabled": `--fg-subtle`, **3.0:1 on a hovered row**, used in **69 places** for group labels,
counts, metadata and every placeholder. The part worth more than the fix is why §2 never caught it
— the four contrast figures that section has carried since kickoff were all true and **all four
were about `--bg-surface`**. A token is read on five backgrounds. **D-63** moves five tokens; the
tool re-checks them, so §2 is now the summary and not the record.

**The focus audit found itself wrong twice before it found anything about the app.** It reported
the one autofocused control on every screen as ringless, because it measured an element that
already had focus; and it **passed the real failure** — the command palette's search field, which
had cancelled the global ring and replaced it with nothing since Phase 2 — because `outline-offset`
still changes when the outline is `none`. A ring that is not drawn, moving. Both mistakes are named
in the tool's own source, because a tool reporting "no findings" is the easiest thing here to
believe and the hardest to check.

**Ticking a checklist is not paperwork, twice over.** Line by line, §10 found `Toggle.svelte`
breaking the radius rule with a hardcoded 10px track since **Phase 0**, and `Dialog.svelte`
trapping focus and closing on Esc since **Phase 2** while never giving focus *back* — so closing
any overlay left the user at the top of the application. The second is exactly what the keyboard
audit's rule 4 is worded against ("the three together"), and it survived because the two visible
parts worked.

**The reference vault exists as a file**, which was the last thing between S-04 and its end-to-end
number. `benchfixture` built it in memory and both S-02 and S-04 are about the *running* app. The
example writes one in two seconds and **reopens it before reporting success**, because this file
exists to be opened.

Green locally: `cargo test --workspace` all suites passing, `cargo clippy --workspace --all-targets
--all-features` clean, `cargo fmt` clean, `svelte-check` 0 errors over 318 files, `prettier --check`
clean, `vite build` clean. `npm run a11y` — **68 surface-audits, no findings**, both audits verified
non-vacuous by breaking a ring and a token on purpose.

**Not done, and deliberately.** S-08's nineteen surfaces, S-04's end-to-end measurement, the first
gate line, the 7-day drive, and R-29's wording. Every one of them wants the app in front of the
author on this desktop, and no CI job can substitute for any of them. That has been the shape of
this phase's tail since the import surface landed, and this session did not change it — it removed
the last work that could be done without the app running.

### 2026-08-08 (the R-29 wording, and the fixture that did not cover what it claimed)

Asked to finish Phase 3. It cannot be finished from here, and saying so first was the session: two
of the three open tasks and four of the six gate lines need the app in front of the author on this
desktop, and one of them needs seven days. What *was* finishable was the third task, because it was
never a build task — the gate's R-29 line has asked since kickoff for "a Bitwarden export covering
**all seven item types**", meaning ours, and no export of Bitwarden's can produce two of them.
Raised as an open question on 2026-08-06 rather than edited, because a gate's own vocabulary is the
author's.

**The author took the proposed re-wording — D-64.** That is where the session was expected to end,
with three documents edited and a box ticked.

**It did not, because acting on the re-wording meant asserting the fixture covered what the new
line claims, and it did not.** `bitwarden-export.json` carried types 1 to 6; `import/bitwarden.rs`
says in its own comment that there are **eight**, and 7 and 8 — the driving licence and the
passport — were in no fixture and no test. D-50's generic path, the one thing standing between a
type nobody here has read about and silent data loss, had been exercised by the bank account
alone. Worse, **the proposed re-wording itself said "all seven of Bitwarden's"**: our number
carried across into a sentence about theirs, written on the day the problem was found, by the same
reading that produced the wrong line in the first place. Nothing re-checks a proposal between the
day it is written and the day it is taken.

So the fixture has both types now, one of them carrying a `folderId` so the folder-merge test runs
over a generic-path item rather than only over mapped ones, and the claim is a test —
`the_fixture_covers_every_bitwarden_item_type`, asserting the set of `type` values equals 1..=8.
Three count assertions moved with it (7 items to 9, and the merged-tag count 3 to 4), and
`nothing_in_the_export_is_dropped_in_silence` passed the two new items unchanged, which is what it
exists to do.

**`cargo fmt --check` was already failing when the session opened**, on
`examples/reference-vault.rs` from the previous commit — Phase 2's own lesson ("`cargo fmt` belongs
after the last file is written") landing on the last file written, and it would have failed CI on
the phase PR rather than on anything anyone was looking at. Fixed here.

Green after the change: `cargo test --workspace`, `cargo clippy --workspace --all-targets`,
`cargo fmt --all --check`, `svelte-check` (318 files, 0 errors), `prettier --check`.
`import_bitwarden` is 18 tests, one more than it was.

Phase 3 is **37/39**, gate **2 of 6**.

**Not done, and not doable from here.** S-08's nineteen surfaces with the pointer physically
unplugged, S-04's end-to-end number, the functional gate line D-38 moved here from G-B′, and the
7-day drive — which is calendar time and cannot start until the author's own passwords are in a
vault. The first three are **one sitting**, and next actions now says so in the order that makes it
one: reference vault, unplug the mouse, walk the nineteen rows, read the palette's performance
marks while it is open, and run the functional line last on the author's own vault, because that is
the vault the seven days then start on.

### 2026-08-08 (the build S-04 would have been measured on)

Opened by the author asking what is left in Phase 3 and to help finish it. The honest answer was
the one the previous session had already written down — two tasks, four gate lines, all of them
needing the app in front of a person — so this session's job was to make that sitting as short as
possible and to check that nothing in it was set up to produce a wrong answer.

**One of them was. S-04's procedure never said which build the number comes from**, and it reads
as a detail until you notice there is only one build in this project with a devtools console:
`tauri dev`, which is also the only launch path any document describes (`npm run dev:app`,
D-16/D-30). The workspace manifest optimizes **dependencies** in the dev profile and deliberately
leaves our own crates unoptimized, so the palette matching that `cargo bench --bench search`
reports at **0.85 ms p95** costs **6.63 ms p95** there — measured this session with
`cargo bench --bench search --profile dev` rather than assumed, because the multiplier was the
whole question. Twelve per cent of a 50 ms budget, spent by a build no user runs.

What makes it worth a feature rather than a note in the phase document: the number would have
looked right. It would have been under budget, written into the gate as evidence, and
indistinguishable afterwards from a measurement of the shipping product — the same shape as
`--fg-subtle` measured on one background, as the palette searching a client-side copy, as
`ipc_audit.rs`'s hand-written module list. **D-65** is the fix: `measure`, a non-default feature
on `src-tauri` that turns on `tauri/devtools`, with `open_devtools()` called in `setup` so the
console is up on launch rather than hunted for in a per-platform context menu. That call is also
the compile-time proof the feature still reaches Tauri — `open_devtools` exists only under
`debug_assertions` or `tauri/devtools` — so a `measure` build that stopped enabling devtools fails
to compile instead of launching without a console.

**The cost is that devtools in a shipped password manager is an inspector on a process holding
decrypted secrets**, so the feature is opt-in and CI now asserts it stays that way: no default
features in `src-tauri/Cargo.toml`, and no workflow passing it. The second grep carries a bracket
in its own pattern (`[m]easure`), because without it the check finds its own source and fails on
it — noted in the step rather than left as a puzzle for whoever edits it next.

**One thing found while reading the instrument, not fixed and not a defect.** The palette drops a
keystroke's `performance.measure` when a later keystroke supersedes it before the results render
(`asked !== issued`), which is correct — nothing rendered, so nothing to time — and it means the
dropped samples are exactly the slow ones. At a release build's speed supersession should be rare,
but the p95 is optimistic by construction, so the procedure now says to read the entry count
before trusting the percentile.

**`open_devtools()` is verified rather than assumed, and it could not be verified the obvious
way.** The inspector does not open as a window of its own — WebKitGTK attaches it inside the
application window — so `xwininfo -root -tree` shows one window for both builds and a screen
capture is refused by the desktop's portal. What separates them is a process: the inspector's own
UI is a page, so the `measure` build runs **two** `WebKitWebProcess` and two
`WebKitNetworkProcess` where the default build runs one of each. Both binaries were launched under
`GDK_BACKEND=x11` and counted. It also settles a question the feature raised without answering:
an inspector *inside* the window is a pane the keyboard walk would Tab through, which is why the
sitting takes two binaries and the nineteen rows belong on the default one.

**The rest of the session was preparation, which is all the repo can now contribute.** Both
release binaries and the reference-vault file are produced ahead of the sitting — `trustvault`,
`trustvault-measure`, `/tmp/reference.tvault` — so the author's hour is spent walking surfaces
rather than waiting on `lto = true`, and next actions 15 and 18 carry the exact commands, the
order, and which binary answers which line.

Phase 3 is **37/39**, gate **2 of 6** — unchanged, and correctly so: nothing here ticks a box.

### 2026-08-08 (the profile the design had drawn and nothing was behind)

Opened by the author with two observations about the UI against `TrustVault App.html`: the user
profile at the bottom left of the sidebar should have a **dropdown menu**, and Settings still did
not match. Both were true and they were true in different ways, which is most of what this session
was about.

**The dropdown was a plain gap.** The prototype's footer opens a 216px popover — five rows, a
header with an avatar and an e-mail, one separator above *Lock vault*. Ours called `onvaults` and
went straight into the vault switcher, so *Settings* and *Lock vault* had no home in that corner
and the row's chevron pointed right, which is the direction that means "goes somewhere else". No
decision covered this; it was never built.

**The content was a decision, and the decision had answered the wrong question.** On 2026-08-04 the
footer became the *vault* footer and Settings' Profile card became the Vault card, because D-03 put
sync out of scope and there is no account — so an avatar, a name and an e-mail would have been
three invented fields. That reasoning is correct. What it never considered is that the choice was
not between displaying a real person and inventing one: it was between those two and **letting the
user name one**. A stored profile is not invented. The author chose that, so **D-70** is the third
option built.

**Where it lives is the part worth defending.** `name` and `email` go into the **sealed body**
(`vault-format.md` §6.6), not beside the settings in D-33's plaintext store. A name and an e-mail
address identify a person, so they belong in the file whose whole purpose is being unreadable
without a key. The cost is stated rather than engineered around — they cannot be read while locked,
so `vault_status.profile` is `null` there and the lock screen still names the vault, not its owner.
`null` and `{ name: '', email: '' }` are deliberately different answers: the first is "locked, so
unknown" and the footer falls back to the vault; the second is "nobody has filled this in", which
is what **every** vault says until somebody does, because onboarding's three steps do not ask.

Four properties keep it from drifting into an account, and each is a test rather than an intention:
nothing authenticates against it; nothing validates it (an address with no `@` is legal — it labels
a recovery kit, it does not receive mail); an **untouched profile writes no key at all**, so every
vault that predates this field encodes to exactly the bytes it did before and the known-answer
vectors stayed valid without being regenerated; and **an unchanged profile is not a save**, because
the dialog's Save is pressed whether or not anything was typed and a write per press would
re-encrypt and atomically replace the whole vault file to store the strings it already held. That
last one is asserted by modification time rather than by bytes — every save draws a fresh nonce, so
equal ciphertext could never have been the check.

*Sign out of TrustVault* and *Lifetime license · Manage* are **deleted rather than drawn inert**.
There is nothing to sign out of and nothing to manage. That is the D-49/D-55/D-61 rule for the
fifth and sixth time, which by now is the rule and not a series of incidents.

**Two things came out of the build that nobody asked for.**

**⌘, is now bound.** The popover prints it beside *Settings*, and a shortcut drawn on screen that
nothing listens for is D-61's defect for a tenth of the price. It is matched on `event.key` rather
than the lowercased copy — `,` has no case, and the shifted character on that key differs by
layout.

**The a11y audit returned a real contrast finding, on a defect older than this session.** The
footer avatar is brass on `--accent-wash`, and the wash is translucent — so the row tinting to
`--bg-hover` changes what the initials are read against: **4.09:1**, under §9's 4.5. The row has
tinted on hover since Phase 2. It was invisible for two phases because `element.focus()` cannot
hover, and D-70's `.open` state is the first version of that background the audit can reach.
`--accent-chip` is the fix, in the D-63 shape: the wash flattened onto `--bg-base`, so the chip
stops moving when the row under it does. `color-mix` was the first attempt and is unusable here —
it computes to `color(srgb …)`, which `scripts/audits/contrast.js` reads as `rgb(1, 1, 1)`, turning
one finding into eighteen false ones. That is the audit becoming useless, not a cosmetic problem,
and it is worth remembering the next time a derived colour looks like the tidy answer.

**Settings keeps its own grouping**, on the author's call. The prototype's General → Security →
Appearance was not adopted; what changed is that the design's Profile card is back, above the Vault
card rather than instead of it. Two cards where the prototype has one, because the vault card
answers *which file, how many items, where* — the thing the prototype had nowhere else to put and
this screen still owes.

Three new harness scenarios (`profileMenu`, `editProfile`, `settingsProfile`) and the `profile`
field added to every status fixture, which the fixture's own note demanded: a field added to a DTO
and not added there does not fail anything, it quietly changes what every surface is measured
against. **80 surface-audits, no findings.** `docs/keyboard-audit.md` gains rows **20** and **21**,
both unticked — the count is now 21 against the gate's 15, which widens the reconciliation that
section already owed rather than changing its shape.

Phase 3 is **37/39**, gate **2 of 6** — unchanged, and correctly so: nothing here ticks a box. What
this session changed is the denominator: the walk's table went from 19 rows to **21**, and the two
new ones are the surfaces the author was looking at when they opened the session.
