//! Writes the audit fixture to a real `.tvault` file, and the manifest the packet capture reads.
//!
//! ```text
//! cargo run --release --example audit-vault -p trustvault-core \
//!     --features benchfixture -- /tmp/audit.tvault /tmp/audit-manifest.txt
//! ```
//!
//! # Why this exists, next to `reference-vault`
//!
//! Same shape, opposite fixture, and the reason is the run it is for. `reference-vault` writes the
//! 1 000-item timing floor: every password unique, none of them breached, and a breach check of it
//! is ≈ 400 seconds of network by §6.9's own arithmetic. That is the wrong vault to point a packet
//! capture at, for three separate reasons, and the third is the one that decides it:
//!
//! * **Nothing in it is pwned**, so a capture taken against it cannot satisfy the live half of the
//!   gate's first line — a known-pwned password reported correctly against the real service.
//! * **A thousand distinct values** makes S-07b's "one range request per distinct value" a claim
//!   nobody will finish counting, and it imposes a thousand requests on a free API to prove it.
//! * **Its strings are derived**, so the capture's forbidden-string list would be a pattern rather
//!   than a list, and "no password appeared on the wire" would be a grep for `pw-\d+-xK9`.
//!
//! `auditfixture` is twenty-one items, [`DISTINCT`] = 12 distinct values, and every string in it is
//! a published constant in this repository — including `password`, whose SHA-1 prefix `5BAA6` is
//! the one `tests/fixtures/hibp-range-5BAA6.txt` was trimmed from and whose live count the entry
//! check measured. So one run against this vault produces a countable number of requests, a real
//! breach hit, and an exact list of the strings that must not appear anywhere in the capture.
//!
//! # The manifest is the point, not a convenience
//!
//! The gate line reads *"only a 5-character SHA-1 prefix leaves the machine — no full hash, no
//! password, no item title, no vault or item identifier"*. Every one of those is a **negative**
//! claim about bytes, and a negative claim checked by a person typing strings into a grep is a
//! claim that quietly narrows to whatever they remembered to type. This program writes the whole
//! list — every password, every title, every username, every item id, every full SHA-1 in both
//! hex cases and in binary, and the vault's own file name — and `scripts/capture-report.py`
//! searches the capture for all of them. The list is generated from the same table the vault is
//! built from, so it cannot fall behind the fixture.
//!
//! It also writes the twelve prefixes that are **permitted** to leave. They are not asserted in
//! the capture — TLS means the request body is not readable, which is D-89 — but the run is not
//! reproducible without knowing what the correct answer looked like.
//!
//! # What it is not
//!
//! A vault whose master password is a published constant, behind the same feature flag as every
//! other fixture and for the same reason: the shipped library does not compile `benchfixture`, so
//! nothing in TrustVault can produce one of these by accident. It refuses to overwrite either
//! path, for D-62's reason — a fixture writer pointed at a real vault is one keystroke away.
//!
//! KDF parameters are the crate defaults rather than [`KdfParams::TESTING`], because a person
//! unlocks this one in the running application.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sha1::{Digest as _, Sha1};
use trustvault_core::auditfixture::{
    DISTINCT, ENTRIES, ITEMS, PASSWORD, PASSWORD_FIELDS, audit_vault,
};
use trustvault_core::{KdfParams, Vault, breach_queries};

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let (Some(vault_argument), Some(manifest_argument)) = (arguments.next(), arguments.next())
    else {
        eprintln!("usage: audit-vault <path.tvault> <manifest.txt>");
        return ExitCode::FAILURE;
    };

    let vault_path = PathBuf::from(vault_argument);
    let manifest_path = PathBuf::from(manifest_argument);
    for path in [&vault_path, &manifest_path] {
        if path.try_exists().unwrap_or(true) {
            eprintln!(
                "{} already exists — refusing to write over it",
                path.display()
            );
            return ExitCode::FAILURE;
        }
    }

    eprintln!(
        "building {ITEMS} items, {PASSWORD_FIELDS} password fields, {DISTINCT} distinct values…"
    );
    let (mut vault, recovery) = match audit_vault(KdfParams::default()) {
        Ok(built) => built,
        Err(error) => {
            eprintln!("could not build the fixture: {error}");
            return ExitCode::FAILURE;
        }
    };

    let manifest = manifest(&vault, &vault_path);

    if let Err(error) = vault.save_to(&vault_path) {
        eprintln!("could not write {}: {error}", vault_path.display());
        return ExitCode::FAILURE;
    }

    // Read it back before saying it worked — `reference-vault`'s rule, and this file exists to be
    // opened by something else for even longer than that one does.
    match Vault::open_file(&vault_path, PASSWORD) {
        Ok(reopened) if reopened.items().count() == ITEMS => {}
        Ok(reopened) => {
            eprintln!(
                "wrote {} but it reopened with {} items, not {ITEMS}",
                vault_path.display(),
                reopened.items().count()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!(
                "wrote {} and could not reopen it: {error}",
                vault_path.display()
            );
            return ExitCode::FAILURE;
        }
    }

    if let Err(error) = std::fs::write(&manifest_path, manifest) {
        eprintln!("could not write {}: {error}", manifest_path.display());
        return ExitCode::FAILURE;
    }

    println!("{}", vault_path.display());
    println!("manifest: {}", manifest_path.display());
    println!("password: {PASSWORD}");
    // Not a secret: the password above is in the source of the crate that generated it.
    println!("recovery: {}", recovery.display().as_str());
    println!("distinct values (= range requests a full check makes): {DISTINCT}");
    ExitCode::SUCCESS
}

/// The capture manifest: what must never appear on the wire, and what is allowed to.
///
/// Line-based `key<TAB>value` rather than JSON, because the only reader is a stdlib Python script
/// and a format a person can also read with `cat` is one they can also check by eye when the
/// report says something surprising. Every value is a published constant of this repository
/// except the item ids, which are freshly generated per vault and are exactly why the manifest is
/// written by the program that writes the vault rather than committed beside it.
fn manifest(vault: &Vault, vault_path: &Path) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# TrustVault packet-capture manifest — R-25, phase 4's gate"
    );
    let _ = writeln!(
        out,
        "# Generated by `cargo run --example audit-vault -p trustvault-core --features benchfixture`."
    );
    let _ = writeln!(
        out,
        "# `forbid` and `forbid-hash` must appear nowhere in the capture; `prefix` is what may."
    );
    let _ = writeln!(out, "vault\t{}", vault_path.display());
    let _ = writeln!(out, "master\t{PASSWORD}");
    let _ = writeln!(out, "items\t{ITEMS}");
    let _ = writeln!(out, "password-fields\t{PASSWORD_FIELDS}");
    let _ = writeln!(out, "distinct\t{DISTINCT}");

    // The prefixes come from `breach_queries` rather than from a second hash of the table, so the
    // manifest is wrong in the same direction as the application if it is ever wrong at all.
    let mut prefixes: Vec<String> = breach_queries(vault)
        .iter()
        .map(|query| query.prefix().to_owned())
        .collect();
    prefixes.sort_unstable();
    for prefix in &prefixes {
        let _ = writeln!(out, "prefix\t{prefix}");
    }

    // The file name, not the whole path: a capture taken in a directory whose name is on the wire
    // for unrelated reasons would be a false positive, and the leaf is the part R-25 means by
    // "vault identifier".
    if let Some(name) = vault_path.file_name().and_then(|name| name.to_str()) {
        let _ = writeln!(out, "forbid\t{name}");
    }

    let mut forbidden: Vec<String> = Vec::new();
    let mut hashes: Vec<String> = Vec::new();
    for entry in ENTRIES {
        forbidden.push(entry.title.to_owned());
        forbidden.push(entry.username.to_owned());
        if !entry.password.is_empty() {
            forbidden.push(entry.password.to_owned());
            let digest: [u8; 20] = Sha1::digest(entry.password.as_bytes()).into();
            hashes.push(digest.iter().map(|byte| format!("{byte:02X}")).collect());
        }
    }
    for item in vault.items() {
        forbidden.push(item.id.to_string());
    }
    forbidden.sort();
    forbidden.dedup();
    hashes.sort();
    hashes.dedup();

    for value in forbidden.iter().filter(|value| !value.is_empty()) {
        let _ = writeln!(out, "forbid\t{value}");
    }
    for hash in &hashes {
        let _ = writeln!(out, "forbid-hash\t{hash}");
    }
    out
}
