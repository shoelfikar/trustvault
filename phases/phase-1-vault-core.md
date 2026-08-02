# Phase 1 — Vault core

> **Provisional.** Written at kickoff as a forecast, not a plan. Run the entry check below before
> the first task, then delete this notice. — *Delete these two lines once the phase is current.*

Status: not started
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

The `trustvault-core` crate in full: the `.tvault` binary format, Argon2id key derivation,
XChaCha20-Poly1305 AEAD, the item model for all seven types, CRUD, atomic save, and recovery-kit
wrap/unwrap. No UI, no Tauri, no clipboard. Requirements covered: R-01…R-07, N-01, N-02, N-03, N-05,
N-09.

This phase carries the project's only Tier-1 code. A defect here is unrecoverable data loss, which
is why the exit gate asks for evidence rather than passing tests.

## Entry check — before the first task

Run once, on the day this phase becomes current, and record the result in the session log of
`trustvault-state.md`.

- [ ] **Still in scope** — traceable to the scope in `trustvault-project.md`, nothing drifted into
      out-of-scope
- [ ] **Requirements still live** — R-01…R-07, N-01, N-02, N-03, N-05, N-09 still exist, still
      wanted, not already satisfied
- [ ] **Dependencies passed their gates** — checked against the gates table in `trustvault-state.md`,
      not against ticked boxes. Depends on: Phase 0 gate
- [ ] **External dependencies met** — none
- [ ] **Task list re-checked** against what Phase 0 actually taught
- [ ] **Exit gate still measurable** as written

Entry check completed: —

## Exit gate

- [ ] `docs/vault-format.md` describes the byte layout well enough that a second implementation
      could be written from it alone — R-02
- [ ] Known-answer test vectors committed (password, salt, params, expected ciphertext) so a future
      refactor cannot silently change the format
- [ ] Mutation test flips every byte of a fixture vault; **every** mutation fails to decrypt — R-04
- [ ] Wrong password and corrupt file are indistinguishable in error type and in timing (±5 %) — R-03
- [ ] 100 injected process kills during save leave a readable vault every time — R-05
- [ ] `cargo llvm-cov` ≥ 90 % on the crate — N-03; `cargo audit` clean; zero `unsafe` — N-05
- [ ] The Argon2id parameter open question is closed with **measured** numbers, per platform

## Tasks

### Format

- [ ] `docs/vault-format.md` written first, before the implementation — R-02
- [ ] Header struct: magic, version, KDF params, salt, nonce; serialized and authenticated as AEAD
      associated data — R-02
- [ ] Body serialized with CBOR (`ciborium`), unknown fields preserved on re-save — N-09
- [ ] Format version constant plus a migration hook that exists and is exercised by a test, even
      though v1 has nothing to migrate from

### Crypto

- [ ] Argon2id KDF with parameters read from the header, never from a constant — R-02
- [ ] Calibration routine: at vault creation, pick m/t/p to hit a target wall-clock time on *this*
      machine and store the result — S-03
- [ ] XChaCha20-Poly1305 AEAD; nonce from the OS CSPRNG on every save, never a counter — R-06
- [ ] Recovery kit: six 4-character base32 groups that wrap the master key independently of the
      master password — R-07
- [ ] Every key type wrapped in `Zeroizing`/`Secret`, zeroized on drop — N-01
- [ ] Constant-time comparison anywhere a secret is compared

### Item model

- [ ] All seven item types with their fields, tags, status, and version history — R-01
- [ ] Field-level `secret` flag, carried in the model rather than inferred from the label
- [ ] CRUD over the in-memory vault
- [ ] Atomic save: temp file in the same directory → fsync → rename — R-05

### Verification

- [ ] Round-trip property test over 10 000 generated vaults — R-01
- [ ] Byte-mutation test — R-04
- [ ] Timing test for the wrong-password/corrupt-file equivalence — R-03
- [ ] Kill-during-save test harness, 100 iterations — R-05
- [ ] KAT vectors committed with a test that reads them
- [ ] `cargo llvm-cov` wired into CI with the 90 % threshold enforced — N-03
- [ ] CI check that `trustvault-core`'s dependency tree contains no Tauri and no UI crate — N-02

Total: 0/21. Removing a task from the *current* phase needs a line in the decision log; removing one
from a phase still marked provisional does not.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Vault core crate | `crates/trustvault-core/` |
| Format specification | `docs/vault-format.md` |
| Known-answer vectors | `crates/trustvault-core/tests/vectors/` |
| Measured Argon2id parameters | `trustvault-requirements.md` S-03, and the decision log |

## Notes

Write `docs/vault-format.md` before the code. A format designed in the spec is a format; a format
extracted from an implementation afterwards is a description of whatever the implementation happened
to do.
