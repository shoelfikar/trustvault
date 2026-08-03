//! Known-answer vectors: committed vault files with their passwords and their expected
//! plaintext.
//!
//! Everything else in this suite tests the implementation against itself — a round trip passes
//! just as happily if both halves of the format changed together. These files were sealed
//! once, on a date, by a build that existed then, and they are the only artefact a refactor
//! cannot argue with. If they stop opening, the format changed, and the only question left is
//! whether `FORMAT_VERSION` was bumped.
//!
//! Regenerate with:
//!
//! ```text
//! cargo test -p trustvault-core --test known_answer -- --ignored --exact generate_vectors
//! ```
//!
//! and then read the diff. A vector file that changes without a version bump is the bug this
//! test exists to catch, so a regeneration is a deliberate act, never a fix.
// Clippy's `allow-unwrap-in-tests` only reaches code inside a `#[cfg(test)]` module. An
// integration test is its own crate, so the crate-level Tier-1 lints in Cargo.toml apply here
// with full force and every `unwrap` in a fixture is an error. The lints stay where they
// matter — library code — and are lifted here, where a panic *is* the failure report.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use trustvault_core::{
    Error, FORMAT_VERSION, FieldKind, Header, ItemKind, ItemStatus, KdfParams, RecoveryCode, Vault,
    VaultBody,
};

/// Serializable copy of [`KdfParams`], which is not itself `Serialize` — the parameters travel
/// in the vault header, not in JSON, so the crate has no reason to expose that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct KdfRepr {
    m_cost: u32,
    t_cost: u32,
    p_cost: u8,
}

impl From<KdfParams> for KdfRepr {
    fn from(params: KdfParams) -> Self {
        Self {
            m_cost: params.m_cost,
            t_cost: params.t_cost,
            p_cost: params.p_cost,
        }
    }
}

impl From<KdfRepr> for KdfParams {
    fn from(repr: KdfRepr) -> Self {
        Self {
            m_cost: repr.m_cost,
            t_cost: repr.t_cost,
            p_cost: repr.p_cost,
        }
    }
}

/// What one vector claims about its `.tvault` file.
#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    /// Why this vector exists — what it would catch.
    description: String,
    /// Format version the file was written with.
    format_version: u16,
    /// The master password, verbatim.
    password: String,
    /// The recovery code in its printed form.
    recovery_code: String,
    /// The parameters the file's header should carry.
    kdf: KdfRepr,
    /// The exact plaintext the file must decrypt to.
    body: VaultBody,
}

fn vectors_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

fn manifests() -> Vec<(String, Manifest)> {
    let dir = vectors_dir();
    let mut found: Vec<(String, Manifest)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("no vectors at {}: {error}", dir.display()))
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .map(|entry| {
            let path = entry.path();
            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_owned();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
            let manifest: Manifest = serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("parsing {}: {error}", path.display()));
            (name, manifest)
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

#[test]
fn every_vector_decrypts_to_its_recorded_plaintext() {
    let vectors = manifests();
    assert!(
        vectors.len() >= 3,
        "expected at least three vectors, found {}",
        vectors.len()
    );

    for (name, manifest) in vectors {
        let path = vectors_dir().join(format!("{name}.tvault"));
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("{name}: reading {}: {error}", path.display()));

        // The header says what it should, before anything is decrypted.
        let header =
            Header::parse(&bytes).unwrap_or_else(|error| panic!("{name}: header: {error}"));
        assert_eq!(header.version, manifest.format_version, "{name}: version");
        assert_eq!(
            KdfRepr::from(header.kdf),
            manifest.kdf,
            "{name}: parameters"
        );

        // The password opens it and the plaintext is exactly what was recorded.
        let opened = Vault::open(&bytes, &manifest.password)
            .unwrap_or_else(|error| panic!("{name}: the password no longer opens it: {error}"));
        assert_eq!(opened.body(), &manifest.body, "{name}: plaintext");

        // So does the recovery kit, independently (R-07).
        let recovery = RecoveryCode::parse(&manifest.recovery_code)
            .unwrap_or_else(|error| panic!("{name}: recovery code: {error}"));
        let recovered = Vault::open_with_recovery(&bytes, &recovery)
            .unwrap_or_else(|error| panic!("{name}: the recovery kit no longer opens it: {error}"));
        assert_eq!(
            recovered.body(),
            &manifest.body,
            "{name}: recovery plaintext"
        );

        // And nothing else does.
        assert!(matches!(
            Vault::open(&bytes, &format!("{}x", manifest.password)),
            Err(Error::Unreadable)
        ));
    }
}

#[test]
fn every_vector_file_has_a_manifest_and_the_other_way_round() {
    // A `.tvault` with no `.json` is a file nobody checks; a `.json` with no `.tvault` is a
    // claim about nothing. Both are how a vector set rots.
    let dir = vectors_dir();
    let mut vaults: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("vectors dir").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "tvault") {
            vaults.push(
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or_default()
                    .to_owned(),
            );
        }
    }
    vaults.sort();

    let described: Vec<String> = manifests().into_iter().map(|(name, _)| name).collect();
    assert_eq!(vaults, described);
}

/// Writes the vectors. Not part of the suite — see the module docs.
#[test]
#[ignore = "regenerates the committed known-answer vectors; run deliberately and read the diff"]
fn generate_vectors() {
    let dir = vectors_dir();
    std::fs::create_dir_all(&dir).expect("create vectors dir");

    write_vector(
        "v1-empty",
        "A vault with no items. Pins the smallest possible body and the header layout.",
        KdfParams::TESTING,
        "correct horse battery staple",
        |_vault| {},
    );

    write_vector(
        "v1-all-kinds",
        "One item of each of the seven kinds, with a secret and a non-secret field. Pins the \
         serialized discriminants, which are part of the format.",
        KdfParams {
            m_cost: 1024,
            t_cost: 2,
            p_cost: 1,
        },
        "a different password",
        |vault| {
            for (index, kind) in ItemKind::ALL.into_iter().enumerate() {
                let id = vault.add_item(kind, format!("Item {index}"));
                let item = vault.item_mut(id).expect("just added");
                item.set_field("Username", format!("user{index}"), false);
                item.set_field("Password", format!("secret-{index}"), true);
                item.tags.push("pinned".to_owned());
                item.status = ItemStatus::Strong;
                if let Some(field) = item.fields.first_mut() {
                    field.kind = FieldKind::Username;
                }
            }
        },
    );

    write_vector(
        "v1-unicode",
        "Emoji, combining marks, right-to-left text, and an embedded NUL. Pins that the format \
         stores text as given and normalizes nothing.",
        KdfParams::TESTING,
        "pässwörd — 🔐",
        |vault| {
            let id = vault.add_item(ItemKind::Note, "🔐 Café الرقم");
            let item = vault.item_mut(id).expect("just added");
            item.set_field("Value", "a\u{0301}b\u{0000}c 🇮🇩", true);
            item.set_field("Empty", "", false);
            item.tags.push("العربية".to_owned());
        },
    );

    println!("vectors written to {}", dir.display());
}

fn write_vector(
    name: &str,
    description: &str,
    params: KdfParams,
    password: &str,
    fill: impl FnOnce(&mut Vault),
) {
    let (mut vault, recovery) = Vault::create(name, password, params).expect("create");
    fill(&mut vault);
    let bytes = vault.to_bytes().expect("seal");

    let manifest = Manifest {
        description: description.to_owned(),
        format_version: FORMAT_VERSION,
        password: password.to_owned(),
        recovery_code: recovery.display().to_string(),
        kdf: params.into(),
        body: vault.body().clone(),
    };

    let dir = vectors_dir();
    std::fs::write(dir.join(format!("{name}.tvault")), &bytes).expect("write vault");
    std::fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_string_pretty(&manifest).expect("serialize manifest") + "\n",
    )
    .expect("write manifest");
}
