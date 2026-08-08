//! Writes the reference vault to a real `.tvault` file, so the app can open it.
//!
//! ```text
//! cargo run --release --example reference-vault -p trustvault-core \
//!     --features benchfixture -- /tmp/reference.tvault
//! ```
//!
//! # Why this exists
//!
//! `benchfixture::reference_vault` builds the 1 000-item vault every timing in
//! `trustvault-requirements.md` is measured against, and it builds it **in memory**. That was
//! enough for the half of S-04 that runs in Rust — `cargo bench --bench search` reported 0.85 ms
//! p95 for the matching itself — and it is not enough for the other half. The criterion is
//! *keystroke to render*, which needs the IPC hop and the webview's paint, which needs the
//! running application, which needs a file on disk. The fixture existed; the file did not.
//!
//! # What it writes, and what it is not
//!
//! A vault whose master password is a **published constant** — `benchfixture::PASSWORD`, printed
//! by this program when it finishes. That is why it is an example behind a feature flag rather
//! than anything the application can reach: the shipped library does not compile
//! `benchfixture`, so nothing in TrustVault can produce one of these by accident.
//!
//! It refuses to overwrite, for `create_vault`'s reason (D-62) and with less excuse than
//! `create_vault` has: a fixture writer that clobbers a path handed to it on the command line
//! is one keystroke from being pointed at somebody's real vault.
//!
//! KDF parameters are the crate defaults rather than `KdfParams::TESTING`. The unlock is not
//! what is being measured, but it is what is being *waited* for, and a fixture that opens
//! instantly measures the palette on a machine behaving differently from the one a user has.

use std::path::PathBuf;
use std::process::ExitCode;

use trustvault_core::benchfixture::{FIELDS_PER_ITEM, ITEMS, PASSWORD, reference_vault};
use trustvault_core::{KdfParams, Vault};

fn main() -> ExitCode {
    let Some(argument) = std::env::args().nth(1) else {
        eprintln!("usage: reference-vault <path.tvault>");
        return ExitCode::FAILURE;
    };

    let path = PathBuf::from(argument);
    if path.try_exists().unwrap_or(true) {
        eprintln!("{} already exists — refusing to write over it", path.display());
        return ExitCode::FAILURE;
    }

    eprintln!("building {ITEMS} items × {FIELDS_PER_ITEM} fields…");
    let (mut vault, recovery) = match reference_vault(KdfParams::default()) {
        Ok(built) => built,
        Err(error) => {
            eprintln!("could not build the fixture: {error}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = vault.save_to(&path) {
        eprintln!("could not write {}: {error}", path.display());
        return ExitCode::FAILURE;
    }

    // Read it back before saying it worked. The whole point of this file is that something else
    // opens it, so a fixture that writes cleanly and does not open is the one failure mode worth
    // spending a second of Argon2 to rule out here rather than in the app.
    match Vault::open_file(&path, PASSWORD) {
        Ok(reopened) if reopened.items().count() == ITEMS => {}
        Ok(reopened) => {
            eprintln!(
                "wrote {} but it reopened with {} items, not {ITEMS}",
                path.display(),
                reopened.items().count()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("wrote {} and could not reopen it: {error}", path.display());
            return ExitCode::FAILURE;
        }
    }

    println!("{}", path.display());
    println!("password: {PASSWORD}");
    // Printed because a vault whose recovery code is unknown cannot be opened the second way,
    // and this file exists to be opened. It is not a secret: the password above is in the
    // source of the crate that generated it.
    println!("recovery: {}", recovery.display().as_str());
    ExitCode::SUCCESS
}
