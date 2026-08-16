# The `.tvault` format, version 1

This document specifies the on-disk format completely enough that a second implementation can be
written from it alone, with no reference to the Rust source. Where this document and
`crates/trustvault-core` disagree, **this document is wrong and must be fixed** — but the
known-answer vectors in `crates/trustvault-core/tests/vectors/` are the tie-breaker, because they are
the only artefact that a refactor cannot silently change.

Requirements referenced here live in `trustvault-requirements.md`. Decisions referenced as `D-nn`
live in the decision log in `trustvault-state.md`.

## 1. Notation

- All integers are **little-endian, unsigned**, and fixed width. There are no varints in the header.
- `[u8; n]` is a fixed-length byte string, stored in order with no length prefix.
- Byte ranges are half-open: `0..4` is four bytes at offsets 0, 1, 2, 3.
- "The AEAD" always means XChaCha20-Poly1305 as specified in
  [draft-irtf-cfrg-xchacha-03](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-03):
  192-bit nonce, 256-bit key, 128-bit Poly1305 tag appended to the ciphertext.
- "The KDF" always means Argon2id, version 0x13 (Argon2 1.3), as specified in
  [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106).

## 2. File layout

A vault file is a fixed 228-byte header followed by one AEAD-sealed body.

```
offset  size  field           notes
------  ----  --------------  --------------------------------------------------------------
     0     4  magic           ASCII "TVLT" (0x54 0x56 0x4C 0x54)
     4     2  format_version  u16. This document specifies 1.
     6     1  kdf_id          u8. 1 = Argon2id v0x13. No other value is defined.
     7     1  aead_id         u8. 1 = XChaCha20-Poly1305. No other value is defined.
     8     4  m_cost          u32, Argon2id memory cost in KiB
    12     4  t_cost          u32, Argon2id time cost (passes)
    16     1  p_cost          u8, Argon2id parallelism (lanes)
    17     3  reserved        [u8; 3], MUST be written as zero, MUST NOT be rejected if non-zero
------  ----  --------------  ----- end of the parameter block (bytes 0..20) ----------------
    20    16  salt_pw         [u8; 16], Argon2id salt for the password key
    36    16  salt_rk         [u8; 16], Argon2id salt for the recovery key
    52    24  nonce_pw        [u8; 24], XChaCha20 nonce for wrap_pw
    76    48  wrap_pw         [u8; 48], master key sealed under the password key (32 + 16 tag)
   124    24  nonce_rk        [u8; 24], XChaCha20 nonce for wrap_rk
   148    48  wrap_rk         [u8; 48], master key sealed under the recovery key (32 + 16 tag)
   196    24  nonce_body      [u8; 24], XChaCha20 nonce for the body
   220     8  body_len        u64, length in bytes of the sealed body, tag included
------  ----  --------------  ----- end of the header (bytes 0..228) ------------------------
   228  body_len  body        sealed body: ciphertext || 16-byte Poly1305 tag
```

The file is exactly `228 + body_len` bytes. Trailing bytes are a corrupt file, not an extension
point — a reader MUST reject a file longer than that (§7).

`reserved` is written as zero and **ignored on read**. It is inside the parameter block, so a
future version can use it without a format break: a v1 reader given a v1 file with non-zero reserved
bytes still decrypts correctly, and a version that gives them meaning bumps `format_version`.

## 3. Keys

Three keys exist. Only the master key ever touches the body.

| Key | Length | Derived from | Lifetime |
|-----|--------|--------------|----------|
| Master key (`MK`) | 32 B | OS CSPRNG at vault creation | The life of the vault, until an explicit key rotation |
| Password key (`KEK_pw`) | 32 B | `Argon2id(password, salt_pw, m, t, p)` | Recomputed on every unlock, zeroized immediately after use |
| Recovery key (`KEK_rk`) | 32 B | `Argon2id(recovery_secret, salt_rk, m, t, p)` | Recomputed only during recovery |

`MK` is wrapped twice, independently, under `KEK_pw` and `KEK_rk`. That is what lets the recovery kit
open the vault without the master password (R-07), and it is why changing the master password
rewrites only `salt_pw`, `nonce_pw`, and `wrap_pw` — the body is not re-encrypted and `MK` does not
change.

`MK` is **never** derived from the password. A password change that re-derived `MK` would require
re-encrypting the whole body, which turns a cheap operation into a long one that can be interrupted.

### 3.1 KDF inputs, exactly

- **Password**: the UTF-8 bytes of the string as the user typed it. **No Unicode normalization, no
  trimming, no case folding.** Normalizing would mean a vault created under one normalization table
  cannot be opened under another, and the tables change.
- **Recovery secret**: the **15 decoded bytes** of the recovery code (§5), not its printed
  characters. Formatting, dashes, and case therefore cannot affect the derived key.
- **Output length**: 32 bytes in both cases.
- **Associated data / secret key** (Argon2's optional `X` and `K` inputs): both empty.

### 3.2 Parameters

Parameters live in the header, are read from the header, and are never taken from a compiled-in
constant at decryption time (R-02). The constants below are only the *starting point* for
calibration on a new vault.

| | Value | Measured |
|---|---|---|
| Default `m_cost` | 262144 KiB (256 MiB) | — |
| Default `t_cost` | 3 | — |
| Default `p_cost` | 1 | — |
| Wall clock at defaults, dev machine | 511 ms | 2026-08-02, Ubuntu 26.04, x86-64, 8 threads, median of 5 after 2 warm-ups |

That satisfies S-03 (≥ 500 ms) with little margin, which is the point of calibrating rather than
hard-coding: at vault creation the implementation measures this machine and stores what it found.

**Calibration algorithm** (normative, so that two implementations agree on the shape even if they
land on different numbers):

1. Hold `m_cost` at the default and `p_cost` at 1.
2. Time one derivation at `t_cost = 1`.
3. Set `t_cost = max(2, round(600 ms / that time))`.
4. Time a derivation at the chosen `t_cost`. While it is under 500 ms and `t_cost < 16`, increment
   `t_cost` and re-measure.
5. If allocating `m_cost` fails, halve it (floor 65536 KiB = 64 MiB) and restart at step 2.

`p_cost` stays at 1 because this implementation does not run Argon2's lanes in parallel; raising `p`
without threads costs the same wall clock while reducing memory-hardness per lane. An implementation
that *does* use threads may raise it.

### 3.3 Parameter bounds on read

A reader MUST reject a header whose parameters fall outside these bounds, **before** allocating
anything:

| Parameter | Minimum | Maximum |
|-----------|---------|---------|
| `m_cost` | 8 KiB | 4194304 KiB (4 GiB) |
| `t_cost` | 1 | 64 |
| `p_cost` | 1 | 64 |

These bounds are not a security control — a downgraded parameter cannot help an attacker, because
wrong parameters produce a wrong `KEK` and the unwrap simply fails. They exist so that a hostile file
declaring `m_cost = 4 TiB` cannot make the reader allocate itself to death before it discovers the
password is wrong.

## 4. Authenticated encryption

Three seals, all XChaCha20-Poly1305, all with a nonce drawn fresh from the OS CSPRNG (R-06). **No
nonce is ever derived from a counter, a timestamp, or the file contents.** With a 192-bit random
nonce the birthday bound is irrelevant, which is the entire reason for choosing the extended-nonce
variant over AES-GCM (D-06).

| Seal | Key | Nonce | Plaintext | Associated data |
|------|-----|-------|-----------|-----------------|
| `wrap_pw` | `KEK_pw` | `nonce_pw` | `MK` (32 B) | header bytes `0..20` (the parameter block) |
| `wrap_rk` | `KEK_rk` | `nonce_rk` | `MK` (32 B) | header bytes `0..20` (the parameter block) |
| body | `MK` | `nonce_body` | CBOR body (§6) | header bytes `0..228` (the whole header) |

The associated data is what makes the header tamper-evident (R-02):

- Bytes `0..20` — version, algorithm ids, KDF parameters — are authenticated by all three tags.
  Editing any of them makes every seal in the file fail. In particular, an attacker cannot rewrite
  `m_cost` down to make a brute-force cheaper: the tag check fails before the value is of any use.
- Bytes `20..228` — both salts, the nonces, the two wrapped keys, and `body_len` — are authenticated
  by the body tag. The nonces and wraps are *not* covered by the wrap tags because they contain the
  wraps themselves; making them the AAD of the wraps would be circular.
- The **salts are deliberately outside the wrap AAD**, and this is the one part of the design that
  is easy to get wrong in the other direction. If both salts were shared associated data, rotating
  `salt_pw` during a password change would invalidate `wrap_rk` — and a password change cannot
  re-wrap under the recovery code, because the user is not holding it. Nothing is lost: a salt is
  already an input to its own KDF, so editing it derives a wrong key and the unwrap fails, and both
  salts are still covered by the body tag.

Every byte of the file is therefore covered by at least one tag, which is what R-04 asserts and what
the byte-mutation test proves file by file rather than by argument.

### 4.1 Order of operations when saving

1. Serialize the body to CBOR.
2. Draw `nonce_body` from the OS CSPRNG. (If the master password or the recovery code changed in this
   save, draw the corresponding salt and nonce and recompute that wrap first — the wraps are part of
   the body's AAD, so they must be final before step 4. Only the changed credential's wrap is
   touched; the other one is untouched by design, see §4.)
3. Write the 228-byte header with `body_len = ciphertext length + 16`.
4. Seal the body with `MK`, `nonce_body`, and header bytes `0..228` as AAD.
5. Write header and sealed body atomically (§8).

A fresh `nonce_body` on **every** save is mandatory. Reusing a nonce with the same key across two
different bodies is a total loss of confidentiality for the differing plaintext, and the whole point
of the 192-bit nonce is that random draws never need coordinating to avoid it.

## 5. Recovery kit

The printed kit carries **120 bits** of CSPRNG output as six groups of four characters:

```
XXXX-XXXX-XXXX-XXXX-XXXX-XXXX
```

- **Alphabet**: RFC 4648 base32, uppercase — `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`. It contains no
  `0`, `1`, `8`, or `9`, which removes the `0`/`O` and `1`/`I`/`l` transcription failures that a
  printed, hand-typed secret is otherwise guaranteed to hit.
- **Encoding**: 15 random bytes → 24 base32 characters, no padding, split into six groups of four.
- **Decoding**: uppercase the input, discard every character that is not in the alphabet (so dashes,
  spaces, and line breaks are all irrelevant), then require exactly 24 characters and decode to 15
  bytes. A character that is *not* in the alphabet but *is* alphanumeric — `0`, `1`, `8`, `9` — is a
  transcription error and MUST be rejected rather than guessed at. Silently mapping `0` to `O` would
  make two different kits open the same vault.
- The 15 decoded bytes are the KDF input (§3.1). The printed string is never hashed.

The kit is shown exactly once, at vault creation (R-07). It is not stored anywhere in the vault: only
`wrap_rk`, from which the code cannot be recovered.

120 bits is beyond brute force, so running the full Argon2id over it is unnecessary work — it is done
anyway so that there is exactly one derivation path in the implementation. A second, cheaper path is
a second thing to get wrong.

## 6. The body

The body plaintext is a CBOR document (D-10, `ciborium`): a map with text keys, serialized from the
structure below. CBOR was chosen over `postcard` and `bincode` precisely for §6.2.

### 6.1 Structure

```
VaultBody {
  vault_id:   text,          // UUID, stable for the life of the vault
  name:       text,          // display name; the filename is not authoritative
  created_at: int,           // Unix milliseconds, UTC
  updated_at: int,           // Unix milliseconds, UTC
  items:      [Item],
  profile:    Profile,       // §6.6; absent entirely when empty
  audit:      [AuditEntry],  // §6.4; absent entirely when empty
  last_scan_at:         int, // §6.7; Unix milliseconds; absent entirely when never
  last_breach_check_at: int, // §6.7; Unix milliseconds; absent entirely when never
  ...unknown                 // §6.2
}

Profile {
  name:       text,          // §6.6; may be empty
  email:      text,          // §6.6; may be empty, never used to sign in
  ...unknown
}

Item {
  id:         text,          // UUID v4
  kind:       text,          // "login" | "api_key" | "card" | "note" | "wifi" | "ssh_key" | "identity"
  title:      text,
  fields:     [Field],
  tags:       [text],
  status:     text,          // "unknown" | "strong" | "weak" | "reused" | "breached" | "expired"
  favourite:  bool,
  created_at: int,
  updated_at: int,
  history:    [HistoryEntry],
  ...unknown
}

Field {
  id:         text,          // UUID v4, stable across edits so the UI can address one field
  label:      text,
  value:      text,
  kind:       text,          // "text" | "username" | "password" | "url" | "email" | "otp" | "note" | "date"
  secret:     bool,          // §6.3
  custom:     bool,          // §6.5; absent entirely when false
  ...unknown
}

HistoryEntry {
  changed_at: int,           // Unix milliseconds, UTC
  field_id:   text,
  value:      text,          // the *previous* value
}

AuditEntry {
  at:         int,           // Unix milliseconds, UTC
  item_id:    text,          // UUID
  field_id:   text,          // UUID
}
```

`status` is a **cache of the last Watchtower scan**, not a computed truth. It is stored so that the
item list can draw its pips without re-scanning at every open, and it is stale by definition until
the next scan. Nothing may make a security decision from it.

`history` holds previous values of secret fields, which means it is as sensitive as the fields
themselves. It is inside the sealed body and never leaves it.

### 6.2 Unknown fields are preserved (N-09)

Every structure above ends with `...unknown`: any key the reader does not recognize is kept as a raw
CBOR value and **written back unchanged** on the next save. A vault edited by a newer version and
then opened, changed, and saved by an older one must not lose the newer version's data — that is
silent corruption, and it is the failure users notice six months later.

This is the reason the body is CBOR rather than a schema-rigid format: in `postcard` or `bincode`,
an unknown field is a parse error, and the only recovery is refusing to open the file.

Unknown keys are preserved, not merged: a reader never invents a value for a key it does not know.

### 6.3 `secret` is stored, never inferred

Whether a field is secret is a property of the field, carried in the model. It is never inferred from
the label. Inference means a field called `Recovery e-mail` is masked because it contains "recovery",
and a field called `PIN` is not because nobody thought of it. The UI masks exactly the fields whose
`secret` is true.

### 6.4 The audit log

`audit` records reveals when the audit setting is on (R-13, D-31). Three rules, all of which a
second implementation must follow to be compatible:

1. **It holds no secret.** A timestamp and two identifiers per entry and nothing else: not the
   value, not the field label, not the item title. An audit log that quotes what it audited is a
   second copy of the vault with none of the ceremony around it.
2. **An empty log writes no key at all.** A vault that has never recorded a reveal encodes exactly
   as it did before this field existed, which is what keeps the vectors in §10 valid across the
   change. A reader MUST treat an absent `audit` as an empty list, not as an error.
3. **Capped at 1000 entries, oldest dropped first**, enforced when an entry is appended.

Rule 3 has a cross-version consequence worth stating rather than discovering. A future version with
a larger cap will write a longer log; this build does not reject it and does not prune it on read,
but the **first reveal it records truncates the log to 1000**. A reader with a smaller cap than the
writer therefore loses history the first time it is written to. This is a deliberate trade against
the alternative — preserving whatever length was found — which would make the cap unenforceable in
exactly the case it exists for.

The log is inside the sealed body for the same reason `history` is: a record of *which* secret was
read *when* is sensitive on its own, whatever it omits. It is **not tamper-evidence** and must never
be argued as such — anyone who can read it holds the master key and can therefore rewrite it. Its
reader is the vault's owner reviewing their own activity.

### 6.5 `custom` is stored, never inferred, and there is no folder

Two questions that an import forces and the design never had to answer, settled together in
**D-43** because they have the same shape: the model gets one new bit, and no new container.

**Custom fields.** `custom` marks a field the user or an import added, as against one of the item
type's own. It is stored for the reason `secret` is stored (§6.3), one step along. The tempting
alternative — derive it, by asking whether the label is in the type's standard set — fails in both
directions on real data: a login whose password field someone renamed becomes "custom", and an
imported custom field that happens to be called "Username" becomes the login's own. The second
direction is the dangerous one, because it silently merges an imported value into a real credential.

Three consequences a second implementation must match:

1. **Absent when false.** A field that is not custom writes no `custom` key at all, so a vault with
   no custom field anywhere encodes exactly as it did before this key existed — which is what keeps
   the vectors in §10 valid without regenerating them. A reader MUST treat an absent `custom` as
   `false`, not as an error.
2. **Custom fields are never addressed by label.** Setting a field by label searches only
   non-custom fields. Two fields may carry the same label as long as at most one of them is not
   custom.
3. **Duplicate custom labels are legal and are not merged.** Bitwarden permits two custom fields
   with the same name, and merging them on import would be a field dropped in silence — the exact
   failure R-29 refuses to call an import.

**Folders.** There is no `folder` key and there will not be one. A folder is a single-parent
grouping, `tags` is a many-to-many one, and every folder is expressible as a tag while the reverse
is not — so a `folder` key would be a second taxonomy over the same items, with the sidebar then
owing two filters that mean nearly the same thing. On import a folder becomes a tag, and a **nested
path is kept verbatim as one tag string** (`Work/Clients`, not `Clients`): flattening to the leaf
name collides across parents, and splitting into two tags claims a hierarchy that tags do not have.

### 6.6 The profile is a label, not an account — D-70

`profile` holds a name and an e-mail address, and it exists because the design draws a person in two
places: the sidebar footer and the Settings card, both with an avatar of initials. TrustVault has no
account and no sync (D-03), so there was nothing behind those pixels — for two phases they carried
the *vault's* initials, name and file instead, which is honest and is a different thing from what
was drawn. D-70 chose the third option: store the two strings, so that the person on screen is one
the user actually named.

Four properties, and a second implementation must match all four:

1. **It is not a credential and it is not validated.** Nothing authenticates against these strings.
   `email` is not checked for an `@` — its job is to label a profile and a printed recovery kit, and
   refusing a string the user chose for their own label would be the format inventing a rule.
2. **Absent when empty.** A profile whose `name`, `email` and unknown keys are all empty writes no
   `profile` key at all, so a vault that predates this field encodes exactly as it did before — the
   same rule `custom` (§6.5) and `audit` (§6.4) follow, and the reason the vectors in §10 stay valid
   without being regenerated. A reader MUST treat an absent `profile` as two empty strings.
3. **Empty is the starting state.** Onboarding's three steps are vault name, master password and
   recovery kit; none of them asks. Every vault begins with no owner named, and the UI treats that
   as an invitation rather than as missing data.
4. **It is inside the sealed body, and that is the point.** A name and an e-mail identify a person,
   so they belong in the part of the file that is unreadable without a key — not beside the settings
   (D-33), which are plaintext JSON in a config directory. The cost is that they cannot be read while
   locked, which is why the lock screen names the vault and never its owner.

`unknown` on `Profile` counts toward "empty" in the N-09 direction: a profile holding only a key this
build does not recognize is **not** empty, and skipping it on write would drop what a newer version
stored.

### 6.7 The two Watchtower timestamps — R-23…R-26

`last_scan_at` and `last_breach_check_at` say when the local scan and the breach check last
finished, in Unix milliseconds. Added 2026-08-16 with `watchtower_scan`, and permitted without a
version bump by §9: a new key in the body is not a format change.

Four properties, and the first two are the reason there are two keys rather than one:

1. **Two passes, two timestamps.** `status: "strong"` means *clean at the last scan*, and the local
   scan and the breach check can be days apart. A single "last scanned" would let this morning's
   local scan vouch for a breach check that has never run, which is the one direction this cache
   must never fail in.
2. **`last_breach_check_at` is written only by a check that finished** — D-86, and it is the one
   key in the body whose absence is load-bearing rather than incidental. A breach check that
   reached three values out of a thousand writes the breaches it found and leaves this key alone,
   because it is the only part of a check that survives a relaunch: `unchecked` lives in the IPC
   response and is gone with the window. A reader MUST treat absence as "never", never as "clean".
3. **Absent when never.** A vault that has not been scanned writes no key at all, the rule `audit`
   (§6.4) and `profile` (§6.6) already follow, and the reason the vectors in §10 stay valid.
4. **They are the only thing that makes `status: "unknown"` legible.** Without a scan timestamp,
   an item nobody ever scanned and an item Watchtower deliberately did not examine are the same
   value — and both are written: an item with **no password field** keeps the status it had rather
   than being called `strong`, because nothing was examined on it.

## 7. Reading a vault

In order. The order is part of the specification because it is what makes R-03 hold.

1. Read at least 228 bytes. Fewer, or wrong magic, or `kdf_id`/`aead_id` not 1 → **not a vault**.
2. `format_version > 1` → **unsupported version**, reported as such. This is the one case where a
   distinct error is correct: a newer file is not an attack, and telling the user to upgrade is the
   only useful thing to say.
3. Parameters outside §3.3 → **not a vault**.
4. File length ≠ `228 + body_len`, or `body_len < 16` → **not a vault**.
5. Derive `KEK_pw` with the header's parameters.
6. Open `wrap_pw`. **On failure, do not return yet** — substitute a zero master key and continue.
7. Open the body with the master key from step 6.
8. If either step 6 or step 7 failed → **unreadable**. Success requires both.

Steps 6 and 7 are why a wrong password and a corrupted file are indistinguishable (R-03): both spend
one full Argon2id derivation, one wrap-sized AEAD open, and one body-sized AEAD open, and both return
the same error. Returning early at step 6 would make a wrong password measurably faster than a
damaged file, which tells an attacker holding a stolen vault whether their guess was structurally
right.

The distinction that R-03 forbids is *wrong password* vs *corrupt file*. Steps 1–4 answer a different
question — "is this even one of ours" — and are decided before any key material exists.

## 8. Writing a vault (R-05)

Atomically, or not at all:

1. Create `.<final-name>.tmp-<8 random hex>` **in the directory the vault lives in**, so that the
   rename in step 5 cannot cross a filesystem boundary.
2. Write the whole file — header and sealed body.
3. `fsync` the temp file.
4. `rename` the temp file over the destination. On POSIX this is atomic; on Windows the
   implementation uses the platform's replace-file call, which gives the same guarantee.
5. `fsync` the containing directory, so the rename itself survives power loss and not just the
   contents.

A reader therefore never sees a partially written vault: it sees the previous file or the new one.
Step 5 is the step usually skipped, and skipping it means the data is durable but the name pointing
at it is not.

If the process dies at any point, the temp file may remain. It is inert — it is a complete, sealed
vault or a fragment of one, and neither is readable without the password.

## 9. What version 2 would change

Recorded now, so the extension points are honest about what they can absorb.

- A new item `kind` or field `kind` is **not** a format change. Readers keep unknown variants as
  their raw text under §6.2.
- A new key anywhere in the body is **not** a format change (§6.2).
- Changing the KDF or the AEAD *is*: `kdf_id` / `aead_id` get a new value, and `format_version` goes
  to 2. A v1 reader stops at step 1 of §7 and says "not a vault", which is blunt but honest.
- Changing the header layout is a `format_version` bump. The 3 reserved bytes at offset 17 exist so
  that adding a flag does not require one.

## 10. Known-answer vectors

`crates/trustvault-core/tests/vectors/` holds vault files with their passwords, recovery codes, and
expected plaintext, generated once and committed. They use deliberately weak KDF parameters (small
`m_cost`, `t_cost = 1`) so the suite runs in milliseconds — the parameters are in the header, so a
weak vector exercises exactly the same code path as a strong one.

The vectors are what a refactor cannot argue with. If a change makes them fail, the change altered
the format, and the only question left is whether `format_version` was bumped.
