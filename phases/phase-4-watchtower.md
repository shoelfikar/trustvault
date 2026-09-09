# Phase 4 — Watchtower

Status: **implementation complete** — entry check run 2026-08-15, five of six boxes; the sixth is
unticked on purpose for the life of the phase (D-74) rather than waved through. The exit gate is
7 of 7 after the remaining manual evidence was recorded 2026-09-09 by user report.
Work happens on `feature/phase-4-watchtower`, cut from `feature/phase-3-surfaces` rather than from
`development` — Phase 3's gate is 5 of 6 so its 44 commits are unmerged, and Watchtower has nothing
to stand on without them (**D-75**).
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

The audit view backed by real data: zxcvbn strength scoring, cross-item reuse detection, and opt-in
HIBP breach checking using k-anonymity with response padding. Requirements covered: R-23…R-26, and
the measurement of S-07 and S-10.

This is the only phase that adds a network call to an application whose entire premise is that it
does not make any. The gate is written accordingly.

## Entry check — before the first task

- [x] **Still in scope** — traceable to `trustvault-project.md`: line 36 names Watchtower's three
      parts, line 81 carries the egress rule this phase is the single exception to, line 122
      defines the view. Nothing has drifted out of scope and nothing here is out-of-scope work
      wearing a requirement's name
- [x] **Requirements still live** — R-23…R-26, all `must`, all tier 4, **none satisfied**. S-07
      and S-10 are still blank in the measurement table. The Watchtower *screen* exists from D-36
      and is drawn against fixtures, which is chrome to fill rather than a requirement already met
- [ ] **Dependencies passed their gates** — **they have not, and the phase opens anyway. D-74.**
      Phase 3's gate is 5 of 6: the seven-day daily drive has not started, and it is wall clock
      rather than work. The drive runs underneath this phase and the gate line ticks on its own
      day. The risk carried is written into D-74 and is not zero: a defect the drive returns lands
      in this phase's branch, and the further this phase gets, the more of it that defect can
      invalidate. **This box stays unticked for the life of the phase** — it is the record that
      the dependency was skipped deliberately, and it is not the kind of box that gets ticked later
- [x] **External dependencies met** — measured against the live service on 2026-08-15, not read
      off this document. `GET https://api.pwnedpasswords.com/range/21BD1` with `Add-Padding: true`:
      **HTTP 200, unauthenticated, no key, no cost**, `vary: add-padding` in the response headers,
      and `cache-control: no-store` on the padded response. Padding is honoured and measured:
      **2034 rows with the header, 1924 without**, of which **110 carry a zero count** — the
      padding — and the 1924 real suffixes are identical in both. The header is still the whole
      mechanism, and it still works
- [x] **Task list re-checked** against what Phase 3 actually taught — five changes, listed under
      *What Phase 3 taught* below. Two of them are new tasks and one re-opens a row of
      `docs/keyboard-audit.md` that currently passes
- [x] **Exit gate still measurable** as written — **no, twice over, and both are fixed rather than
      waved through. Closed 2026-08-15 by the author, D-77.** Gate line 2 asked packet capture to
      confirm "responses contain 800–1000 rows (padding honoured)"; the live service returns **1 924
      real rows before any padding**, so the band is impossible rather than unmet, and the decoy
      count moved **110 → 125 → 156** for the same prefix inside one day. Gate line 4 asked for a
      full scan including HIBP in **10 s**, against a measured ≈ **400 s**. Both are now split in two
      — what leaves and what returns, the local half and the network half — and the gate is **seven
      lines rather than five**. What acting on it cost, which is D-64's lesson holding a second time:
      re-measuring the service to write the new line found a **third** decoy count, and re-reading
      `benchfixture.rs` to write S-07a found that the reference vault has **no reuse and one repeated
      zxcvbn score**, so it is a timing floor and not a fixture R-23 or R-24 can be measured against

Entry check completed: **2026-08-15**, five of six. The one open box — dependencies passed their
gates — is open on purpose and stays open for the life of the phase (D-74).

## What Phase 3 taught

Five things, and each one changes a task rather than decorating the phase.

**The contract comes before the command.** Phase 3's most valuable habit was extending
`docs/ipc-contract.md` with every command marked `// planned` *before* writing any of them, and
deleting the marker in the same commit as the implementation. `ipc_audit.rs` enforces both
directions. Watchtower's commands — the scan, the breach check, the setting — get the same
treatment, and that is a task now rather than a note.

**The elision rule reaches this phase in a way the task list did not say.** Reuse detection groups
items by password; the *finding* that crosses IPC must carry item ids and metadata only, never the
value or anything from which the value can be recovered. A grouping key is a hash of a secret, and
a hash of a short secret is a secret. R-10 and the one rule in `CLAUDE.md` both apply, and
`ipc_session.rs` is where that gets proven rather than promised.

**The command budget is a test, not a convention.** `ipc_audit.rs` asserts the sanctioned set is
*exactly four* and that the contract's own sentence still says four, so the budget cannot move
without a decision cited by number. Nothing in Watchtower should need a fifth — a finding is not
a secret — and if something appears to, that is the design being wrong rather than the budget.

**Anything drawn needs harness scenarios on the day it is drawn.** Three audits now run in CI over
every scenario (D-71, D-72), and finding 6 was a fixture missing a field the host had gained —
the Settings surface measured for weeks against an object the host cannot produce. A DTO field
added without a fixture field fails nothing and quietly changes what every surface is measured
against. Adding the fixtures is part of the task that adds the field.

**Row 17 of `docs/keyboard-audit.md` will re-open.** It reads "the findings list is a list: ↑/↓ and
Enter to the offending item", and it passed on 2026-08-14 against a view drawn from fixtures. Real
findings change that surface, and D-67 is the precedent: a row ticked against markup that no longer
exists is worse than an empty one. Re-walking it is a task in this phase, not an oversight to
discover at the gate.

## Exit gate

- [x] A known-pwned password is reported correctly against **both** a mocked endpoint and the live
      HIBP service — R-25. **Both halves met 2026-08-16.** The mocked half is
      `a_known_pwned_password_is_found_in_the_range` against `hibp-range-5BAA6.txt`, a trimmed real
      response. The live half is the author's `on` sitting: the audit fixture's three items
      carrying `password` came back **breached at 52 372 427** — the same count `probe` read out of
      the service by curl the same day, and the same one the offline fixture carries. **An
      observation, not a measurement**, like G-B′'s fifth line and Phase 3's functional line:
      nothing in CI runs the app, so nothing in CI reproduces it. What keeps it from being
      anecdotal is that the number cannot come from anywhere else — the fixture response is
      compiled into the test binary, not the application, so a release build printing it has had an
      answer from `api.pwnedpasswords.com`, which the capture separately shows it asking for by DNS
      and by SNI
- [x] Packet capture confirms that **only a 5-character SHA-1 prefix leaves the machine** — no full
      hash, no password, no item title, no vault or item identifier — R-25. **Tested earlier and
      recorded 2026-09-09 by user report.**
- [x] For the prefix in that capture, the padded response carries **at least one zero-count row** and
      is **larger than the unpadded response for the same prefix** — padding honoured, stated as
      something the service can be held to — R-25. **Re-worded 2026-08-15, D-77.** It read "responses
      contain 800–1000 rows"; the band is impossible rather than unmet, and the row count moved three
      times in one day (110 → 125 → 156 decoys) over 1 924 real rows that never moved. **Tested
      earlier and recorded 2026-09-09 by user report.**
- [x] With breach checking off — the default — `tcpdump` on the app's PID shows **zero** packets
      across 10 minutes of active use — S-10, R-26. **Tested earlier and recorded 2026-09-09 by
      user report.**
- [x] The **local** scan of the reference vault completes within S-07a's budget, measured with the
      network untouched — S-07a. **The budget is set from the first measurement and is blank until
      then** (D-77), the way S-03's 511 ms was; it is a `cargo bench` line, so CI re-runs it.
      **The budget is no longer blank — 500 ms, set 2026-08-15 from a reading of 61–63 ms (D-80) —
      and this line still does not tick.** Ticking it the same day its own budget was derived from
      the measurement would be circular: the reading cannot fail a criterion computed from it. What
      makes it a gate line is a run that could have failed, which needs the scan reachable through
      `watchtower_scan` and the benchmark re-run by a machine that is not the one it was set on.
      **Both conditions are met and this line ticks, 2026-08-16.** The scan has been reachable
      through `watchtower_scan` since that command landed, and the benchmark ran on a GitHub
      runner — **62.5 ms against the 500 ms budget** for 1 000 items, CI run **31937787913**, the
      first time this branch had ever been on a runner. The runner's number and this desktop's
      61–63 ms agree to within a millisecond, which is a stronger result than the budget: the
      8× headroom D-80 wrote in was sized for a machine that might be much slower, and the one
      that turned up was not. The same run also measured the **audit vault** at 2.7 ms for 21
      items, and the two together give a fact neither gives alone — 0.062 ms per password on the
      reference vault against 0.128 ms on the audit one, so **a password zxcvbn's dictionaries
      match costs 2.1× one they do not**. That is the caveat in this line's own wording turned
      into a number: the reference vault is the cheap case, and the multiplier is 2.1, not the
      order of magnitude the caveat left room for
- [x] The **breach check** of the reference vault makes **one range request per distinct value**
      (1 000, not 4 000), holds its concurrency to a bound, reports progress, and leaves the vault
      consistent when interrupted — S-07b. **Re-worded 2026-08-15, D-77**, from "full scan within
      10 s incl. HIBP": that is ≈ 400 s at the rate the live service actually gives, and a stopwatch
      on the network half measures the user's link and HIBP's cache rather than this code. **Tested
      earlier and recorded 2026-09-09 by user report.**
- [x] Capture evidence recorded in the session log of `trustvault-state.md`, not just asserted

→ **G-C** is crossed after this gate: S-01…S-11 in `trustvault-requirements.md` filled in with
measured results before Phase 5 opens.

## Tasks

### Scoring

- [x] zxcvbn integrated in the core, scoring every password in the vault — R-24. **Done
      2026-08-15**, `crates/trustvault-core/src/watchtower.rs`. It was integrated in the *host*
      already, for onboarding's meter; **D-78** moved it, because scoring every password means
      reading every password and the core is where they are. `score_password` is a pass-through
      now, so there is one definition of a score rather than two
- [x] Crack-time estimates rendered in words, matching zxcvbn's own phrasing — R-24. **Done
      2026-08-16**, in the weak group's row note: *"Crackable in 31 minutes"*, the string handed
      through from zxcvbn rather than re-derived. **It replaces the prototype's own copy and that is
      a decision — D-82**: the prototype reads *"10 characters · word + year"*, and a character
      count is the **length of a password**, which D-32 made every mask a fixed width specifically
      to keep off this boundary
- [x] Reuse detection: group items by password hash, report every member of a group of ≥ 2 — R-23.
      **Done 2026-08-15.** R-23's criterion is a test verbatim — three items sharing a password
      are all reported, each naming the other two and not itself. The key is SHA-256 and it never
      leaves the module; `shared_with` carries item ids the list already has
- [x] Weak-password detection thresholds defined by zxcvbn score, not by length. **Done
      2026-08-15, and the number is measured rather than chosen — D-79.** Score **≤ 2**, not the
      ≤ 1 the first draft assumed: `Tr0ub4dour&3` scores 2 and falls in 31 minutes, and this
      view's own copy has said *"Crackable in a matter of hours"* since D-36 drew it. It leaves
      the meter calling that password *Fair* while this screen calls it *Weak* — an open question
      in `trustvault-state.md`, not a silent edit to either surface

### Breach checking

- [x] HIBP range client: SHA-1, first 5 characters, `Add-Padding: true` — R-25. **Done 2026-08-16**,
      `src-tauri/src/hibp.rs`, on **ureq 3.4** after the survey next action 28 asked for — **D-83**,
      and the survey's own first finding was that the premise everyone starts from is false:
      `reqwest` is in `Cargo.lock` but `tauri` declares it for **mobile targets only**, so the
      desktop baseline has no HTTP client and `cargo tree --target all` says otherwise. The hashing
      is in the **core** (D-78's argument again — hashing every password means reading every
      password), and what crosses the crate boundary is `BreachQuery`: a prefix that may leave, a
      suffix with **no accessor at all**, and the ids. `hibp.rs` cannot log the suffix because it
      has no way to obtain it
- [x] Suffix matching done locally against the returned range. **Done 2026-08-16.** The query does
      the comparing, not the response — `Range::count_for` asks `BreachQuery::matches`. Three
      things it is pinned against, each a test: **a zero-count row is padding and never a hit**,
      **CRLF must not survive into a suffix** (the live service sends `\r\n`, and a stray `\r`
      makes every password read as unbreached — R-25's forbidden direction), and a **200 that
      parses to nothing** is a failure rather than "no match"
- [x] Rate limiting and backoff, so a 1000-item vault is a polite caller. **Done 2026-08-16**:
      three attempts, 1 s → 2 s → 4 s, on 429 and 5xx only. A 4xx is an answer and repeating it is
      rude rather than robust, and that is a test too. The `sleep` is injected so the policy is
      provable without a test that waits seven seconds to show it waited
- [x] Opt-in setting, off by default, with copy that says plainly what leaves the machine — R-26.
      **Done 2026-08-16**: `Settings.breach_check_enabled`, false in `Default`, a test on the
      default because that is the line S-10 is measured on, and a Settings row whose copy is the
      requirement rather than a description of it — *the first five characters of each password's
      SHA-1 hash*, never a password, never an item, never anything naming this vault, and *with it
      off, TrustVault makes no network connection at all*. **The setting is read in the command,
      not in `hibp.rs`** (§6.9), so a caller cannot route around it, and with it off the command
      returns before a client is constructed. The proof is D-84's shape rather than a boolean read
      back: a test starts a listener at the only address the client could reach and asserts it is
      never contacted — verified by deleting the check, which reaches it
- [x] Offline and error paths: a failed check reports "not checked", never "safe". **Done
      2026-08-16.** A failed value lands in `unchecked` with `offline` or `http`, its cached status
      is left exactly as the local scan wrote it, and `record_breaches` promotes **nothing** to
      strong — "absent from the range HIBP served us today" is not a verdict about a password. The
      part that is not obvious and is now **D-86**: a partial pass does not stamp
      `last_breach_check_at`, because that timestamp is the only part of a check that survives a
      relaunch, and a pass that reached three values out of a thousand would otherwise have
      tomorrow's reader told this vault was checked. On screen the same rule is one sentence —
      *"N of M passwords could not be checked … those are not checked — not safe"* — carrying the
      failures in the same breath as the findings rather than under them

### View

- [x] Four stat tiles matching the design. **Done 2026-08-16, and where the four numbers come from
      was the decision in it.** They count **items by cached status**, not findings — the same field
      the item list's pips and the sidebar's badge read, so the three can never disagree, and each
      item is counted once under its worst verdict. *Safe* could not come from anywhere else in any
      case: §6.9 gives the report no row saying strong. The version that counted findings was written
      first and rejected for the case that matters, a **refused scan**: fed from a report that never
      arrived, the tiles read *0 breached, 0 reused, 0 weak* above an error message — a clean bill of
      health issued by a check that did not run, which is R-25's "not checked, never safe" one screen
      early. The **groups** below stay per-finding and do overlap, which is where an item that is
      both breached and reused appears twice with a row explaining each
- [x] Breached / Reused / Weak groups with per-row actions routing to the item. **Done 2026-08-16.**
      Rows come from findings, so the reuse rows read *"Same as deploy@production"* — the prototype's
      own copy, and the reason `shared_with` carries ids rather than a group key. **Expired and
      Breached draw no group today**, because nothing produces either verdict: §6.9's rule that the
      surface must not imply a check that is not running, holding by construction rather than by copy
- [x] Status colours never carry meaning alone — icon and label always present (`MASTER.md` §2).
      **Held through the rewrite 2026-08-16**: every stat tile keeps its icon and its word, every
      group header its icon and title, and every row a type glyph, a sentence and an action verb.
      Re-measured by `npm run a11y` over both Watchtower scenarios — contrast, focus and tab order,
      clean in both themes
- [x] Empty state for a clean vault that reads as reassurance, not as a blank screen. **Done
      2026-08-16, and it is a fixture as much as a surface.** The copy existed; nothing could reach
      it, because the only unlocked scenario in the harness is a vault with four flagged items. The
      new `watchtowerClean` scenario is a scanned vault with two strong items and an empty report, so
      *"Watchtower has nothing to report."* is now photographed in both themes and audited like every
      other screen. Three states, not one, and all three have a scenario now: **never scanned**,
      **scanned and clean**, and **a scan the host refused** — the last says so and offers *Try
      again*, which is the one control on this screen the prototype does not draw at all (it draws no
      failure), and the alternative is a dead end until the vault is locked and reopened

### Verification

- [x] Packet capture harness scripted and repeatable. **Done 2026-08-16**, `scripts/capture.sh`
      with `scripts/capture-report.py` under it and `examples/audit-vault.rs` in front of it. Four
      subcommands: `selftest` (no root, no network, in CI), `probe` (the service, with curl),
      `off` and `on` (the two capture runs). What makes it a harness rather than a note:
      - **The forbidden-string list is generated, not typed.** Gate line 2 is four negative claims
        about bytes, and a negative claim checked by a person grepping is a claim that narrows to
        whatever they remembered. `audit-vault` writes the vault *and* a manifest of every
        password, title, username and item id in it, plus each full SHA-1 in upper hex, lower hex
        and **raw binary** — the rendering a hex-only grep misses. The report searches all of them
        across the **whole** capture, not the part attributed to the app
      - **The fixture is the audit vault, not the reference one (D-90)**, so the run makes twelve
        requests rather than a thousand and contains a genuinely pwned password
      - **The app is a process tree, not a process (D-88).** WebKitGTK does its networking in a
        separate `WebKitNetworkProcess`; attribution by the main PID alone would have watched the
        one process least likely to open a socket
      - **What TLS hides is written down rather than glossed (D-89)**, and the report says it in
        its own last section: the five characters are asserted by `hibp.rs`'s listener test, not
        by this
      - **The report script fails on demand.** `--self-test` builds a capture, analyses it, then
        breaks it eight ways — a planted password, a hash in each of its three shapes, a
        ClientHello naming somewhere else, app traffic during an `off` run. It is a CI step, so
        the thing that decides whether the capture passed is itself checked on every push
      What is still owed is the **run**, which needs a person, a build and `sudo` — the gate lines,
      not this box. Gate line 3's half that belongs to the service was measured the same day:
      `probe` returned **2 151 padded rows against 1 978 unpadded, 173 of them zero-count**, and
      the count for `password` at **52 372 427** — the fixture's own number, from the live service
- [x] Mocked HIBP fixtures committed so the test suite works offline. **Done 2026-08-16**,
      `src-tauri/tests/fixtures/hibp-range-5BAA6.txt` — a **trimmed real response**, not a
      synthesized one: the rows, the counts, the CRLF endings and the zero-count padding all came
      off the live service on 2026-08-16, so a parser that only works against something this
      project invented cannot pass against it. It carries the published SHA-1 of `password`
      (`5BAA6` + `1E4C9B93F3F0682250B6CF8331B7EE68FD8`, count **52,372,427**), which makes it the
      **mocked half of the gate's first line**. The suite needs no network and none of it is
      `#[ignore]`d

### Added at the entry check — 2026-08-15

- [x] `docs/ipc-contract.md` carries every Watchtower command, marked `// planned`, **before** the
      first one is written — Phase 3's habit, enforced in both directions by `ipc_audit.rs`.
      **Done 2026-08-15**, §6.9: two commands (`watchtower_scan`, `watchtower_breach_check` —
      D-76), one event, one setting, two `vault_status` timestamps. Marker verified load-bearing by
      deleting it on purpose — the harness fails naming the command. It cost one finding before a
      line of Watchtower code existed: **S-07 is not measurable as written**, in the open questions
- [x] A Watchtower finding is proven elided in `ipc_session.rs`: item ids and metadata cross IPC,
      never a password and never a hash of one — R-10. **Done 2026-08-16**, in the same commit as
      the command it audits. The session adds a **twin item carrying the value the edit stored**,
      because a report with no reuse group has no grouping key in it and the check would have been
      measuring nothing. Two assertions, and the second is the one the existing transcript checks
      could not make: the two `carrying(SECRET)` searches cannot see a **hash**, so the scan's
      crossing is also searched for any run of **64 hex characters** — what a leaked SHA-256 looks
      like whatever a future field calls it. Verified non-vacuous by adding a `group` field to
      `Finding` on purpose: `ipc_audit.rs` fails naming two of them
- [x] Harness scenarios and fixtures for every new Watchtower surface, added with the field rather
      than after it, so the three CI audits measure the real thing (finding 6). **Done 2026-08-16**:
      the two `vault_status` timestamps landed in the fixture with the DTO, `watchtower_scan` is
      answered with a report **built to agree with `ITEMS`** — one item in two groups, a reuse group
      with two members so *"Same as …"* has something after it — and **two** scenarios join the
      twenty: `watchtowerClean` and `watchtowerError`, taking the sweep to twenty-two. Every audit run
      over the three Watchtower scenarios is clean, in both themes
- [x] Row 17 of `docs/keyboard-audit.md` re-walked once the view shows real findings — it passed
      against fixtures on 2026-08-14 and the surface underneath it changed afterward. **The surface
      changed on 2026-08-16 and the row was left un-ticked until the re-walk evidence was recorded**
      (D-67's precedent, and this re-open was predicted at the entry check). What it now walks is real rather than assumed: ↑/↓ between
      findings **did not exist** before today — nothing on that screen answered an arrow key — and it
      is now `ItemList.svelte`'s pattern, every row a real tab stop. **Tested earlier and recorded
      2026-09-09 by user report.**

### Added when the gate was re-worded — 2026-08-15, D-77

- [x] A **scoring and reuse fixture** that is not the reference vault. `benchfixture.rs` gives every
      item `pw-{index}-xK9`: no two items share a password, so R-23 has nothing to group, and every
      password is the same unmatched shape, so R-24 has one score repeated a thousand times. Both
      requirements need a vault built to have groups and a spread of zxcvbn scores in it, and it is
      a separate fixture rather than a change to `benchfixture.rs` — four criteria are already
      measured against that vault and moving it would move them. **Done 2026-08-15**,
      `crates/trustvault-core/src/auditfixture.rs`: 21 items, reuse groups of 3, 2, 5 and 2, a
      password at **every** zxcvbn score, one item storing one value in two of its own fields, and
      one item with no password at all. Every score in the table was **read out of zxcvbn rather
      than guessed**, and a test re-reads all of them on every run, so a dependency bump that
      reshuffles the scoring fails here by name instead of quietly making the spread a fiction.
      One row is a canary for the context argument — `priya.raman.2024` on the item whose username
      is `priya.raman` scores **4 bare and 2 in context**, so dropping `scan`'s context argument
      fails two tests; verified by dropping it on purpose
- [x] `cargo bench` for the local scan, which is what sets **S-07a's blank number**. It runs against
      the reference vault with the network untouched, and the number it produces is written into
      `trustvault-requirements.md` as the target the same day it is first read — the caveat travelling
      with it being that this fixture is zxcvbn's **cheap** case. **Done 2026-08-15, D-80**:
      **61–63 ms** for 1 000 items, budget set at **≤ 500 ms** with every multiplier in the gap
      written down, and the benchmark **fails** over it rather than printing a regret — it is a CI
      step, which is what the gate line already claimed and nothing did. It also scans the audit
      fixture, because the only honest way to carry the cheap-case caveat is a number beside it:
      a password the dictionaries match costs **1.9×** one they do not

### Added when the client landed — 2026-08-16

- [x] **The client's egress rules proven by watching where the bytes would have gone** — no proxy
      from the environment, no plaintext, no followed redirect. A task added rather than found
      ticked, because **no box owned it**: the Verification group's packet capture is the wire at
      the end of the phase, and these are the three settings that decide what the capture would
      even see. **Done 2026-08-16**, `tests/hibp_egress.rs` and two tests in `hibp.rs`, with
      **D-84** for the reason it is a separate task at all — the first version of two of these
      three **passed with the override they guard deleted**. `ALL_PROXY` is process-global and
      `cargo test` runs a binary's tests on threads, which is why the proxy one is its own binary
      rather than racing every other test that builds a client

### Added when the scan crossed IPC — 2026-08-16

- [x] **`watchtower_scan` registered, the status cache written, and `vault_status` carrying the two
      timestamps.** A task added rather than found ticked, because **no box in this document owned
      it**: every task above is worded for the core (*"zxcvbn integrated in the core"*) or for a
      surface (*"crack-time estimates rendered"*), and the boundary between them — the command, the
      cache write, the save — was named in `docs/ipc-contract.md` §6.9 and nowhere in the task list.
      That is the Phase 3 accounting lesson in the other direction: there, host work sat under boxes
      worded for surfaces and stayed unticked; here, work with no box at all would have been done and
      invisible. **Done 2026-08-16**: `src-tauri/src/commands/watchtower.rs`, the `// planned` marker
      deleted in the same commit (the tenth time that habit has held), `last_scan_at` and
      `last_breach_check_at` in the sealed body (`vault-format.md` §6.7, no version bump) and on
      `vault_status`, and **D-81** for the one asymmetry in the cache write — an item with no
      password field keeps the status it had rather than being called `strong`, because nothing on it
      was examined

### Added when the breach check crossed IPC — 2026-08-16

- [x] **`watchtower_breach_check` registered, the worker pool bounded, and the results dropped if
      the vault locked.** A task added rather than found ticked, for the **third** time in this
      phase and for the same reason each time: the two boxes above are worded for a *setting* and
      for *paths*, and the command between them — registration, the concurrency bound, the progress
      event, the two vault acquisitions and the write — was specified in `docs/ipc-contract.md` §6.9
      and in no task. **Done 2026-08-16**: the `// planned` marker deleted in the commit that
      registered it (the eleventh time that habit has held), `record_breaches` in the core beside
      `scan_and_record`, and three decisions the task list had not anticipated — **D-85** (four
      workers, a bound rather than a target, because §6.9's own numbers are eight concurrent for
      1.34× the serial rate), **D-86** (only a complete pass stamps the vault) and **D-87** (two
      more sentences the prototype cannot keep). The rule that took the most care is the one with
      no surface at all: the second vault acquisition does **not** go through `with_vault`, because
      that touches the idle clock and a minutes-long background task that resets the auto-lock timer
      keeps a vault unlocked for as long as it runs. Two harness scenarios came with it —
      `watchtowerChecked` and `watchtowerPartial`, both **driven through the button** rather than
      fed in as state, because a scenario that set the report directly would photograph a screen no
      sequence of clicks can produce — taking the sweep to **twenty-four scenarios and 144
      surface-audits, clean in both themes**

Total: 24/24 — four added at the entry check, two when the gate was re-worded, one when the scan
crossed IPC, one when the client landed, one when the check did. The denominator has now moved
**three times for the same reason**, and the reason is worth more than the number: a task list
written before the code is worded for the layers the author was picturing, and the work that falls
between two layers has no box. Every time, it was found by doing the work and looking for the box
afterwards. **The capture harness is the counter-example and it is worth noting as one**: it grew a
fixture example, a manifest format, a report script and a CI step, and all four sit inside the box
that was already written for them. A task worded for a *tool* survives the work; a task worded for
a layer does not.

The last box, row 17's Watchtower walk, was tested earlier and recorded 2026-09-09 by user report.
Beside it stand the gate's own two capture runs — scripted as of today, and still a person with
`sudo` and ten minutes.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Scoring, reuse detection, and the breach queries | `crates/trustvault-core/src/watchtower.rs` |
| HIBP client | `src-tauri/src/hibp.rs` |
| Both commands, and the four bounds around the second | `src-tauri/src/commands/watchtower.rs` |
| Watchtower view | `src/lib/shell/Watchtower.svelte` — **not** `screens/`, which is where this table said to look until 2026-08-16 |
| The opt-in row | `src/lib/shell/SettingsPane.svelte` |
| Capture evidence | `trustvault-state.md` session log |

## Notes

The HIBP client belongs in `src-tauri`, not in `trustvault-core` — N-02 keeps the core free of I/O,
and a vault-format crate that can open sockets is a vault-format crate that will eventually be asked
to.
