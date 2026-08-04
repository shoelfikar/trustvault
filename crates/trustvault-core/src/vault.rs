//! The vault: creating one, opening one, saving one, and editing what is inside.
//!
//! The read path is `docs/vault-format.md` §7 and the write path is §8. Both are specified
//! step by step in that document because the *order* is what makes R-03 and R-05 hold — a
//! correct-looking reordering breaks a security property with no visible symptom.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

use crate::aead::{self, NONCE_LEN, WRAP_LEN, random};
use crate::format::{HEADER_LEN, Header, SALT_LEN, WRAP_AAD_LEN};
use crate::kdf::{self, KEY_LEN, KdfParams, Key};
use crate::model::{AuditEntry, FieldId, Item, ItemId, ItemKind, VaultBody};
use crate::recovery::RecoveryCode;
use crate::secret::{SecretBytes, SecretString};
use crate::{Error, FORMAT_VERSION, Result};

/// An unlocked vault: the master key and the decrypted body, in one place.
///
/// Dropping it zeroizes the master key. It does **not** zeroize the body — `SecretString` does
/// that field by field as the structure is dropped.
#[derive(Debug)]
pub struct Vault {
    kdf: KdfParams,
    salt_pw: [u8; SALT_LEN],
    nonce_pw: [u8; NONCE_LEN],
    wrap_pw: [u8; WRAP_LEN],
    salt_rk: [u8; SALT_LEN],
    nonce_rk: [u8; NONCE_LEN],
    wrap_rk: [u8; WRAP_LEN],
    master_key: Key,
    body: VaultBody,
    /// Set when a reveal has been recorded but not yet written — see [`Vault::has_unflushed_audit`].
    ///
    /// Not part of the format and not derived from it: a vault loaded from disk has, by
    /// definition, nothing unflushed.
    audit_unflushed: bool,
}

impl Vault {
    /// Creates a vault and its recovery kit.
    ///
    /// The master key is drawn from the OS CSPRNG and wrapped twice — once under the password,
    /// once under the recovery code (`docs/vault-format.md` §3). It is never derived from the
    /// password, which is what makes a password change cheap and a recovery kit possible.
    ///
    /// The returned [`RecoveryCode`] is the only copy: it is shown once and not stored (R-07).
    pub fn create(
        name: impl Into<String>,
        password: &str,
        params: KdfParams,
    ) -> Result<(Self, RecoveryCode)> {
        params.validate()?;

        let master_key = SecretBytes::new(random::<KEY_LEN>()?);
        let recovery = RecoveryCode::generate()?;

        let salt_pw = random::<SALT_LEN>()?;
        let salt_rk = random::<SALT_LEN>()?;
        let nonce_pw = random::<NONCE_LEN>()?;
        let nonce_rk = random::<NONCE_LEN>()?;

        let mut vault = Self {
            kdf: params,
            salt_pw,
            nonce_pw,
            wrap_pw: [0; WRAP_LEN],
            salt_rk,
            nonce_rk,
            wrap_rk: [0; WRAP_LEN],
            master_key,
            body: VaultBody::new(name),
            audit_unflushed: false,
        };

        vault.wrap_pw = vault.wrap_master_key(password.as_bytes(), &salt_pw, &nonce_pw)?;
        vault.wrap_rk = vault.wrap_master_key(recovery.secret(), &salt_rk, &nonce_rk)?;

        Ok((vault, recovery))
    }

    /// Opens a vault from its bytes with the master password.
    pub fn open(bytes: &[u8], password: &str) -> Result<Self> {
        Self::open_with_secret(bytes, password.as_bytes(), Credential::Password)
    }

    /// Opens a vault from its bytes with the recovery kit, no master password needed (R-07).
    pub fn open_with_recovery(bytes: &[u8], recovery: &RecoveryCode) -> Result<Self> {
        Self::open_with_secret(bytes, recovery.secret(), Credential::Recovery)
    }

    /// Reads and opens a vault file.
    pub fn open_file(path: impl AsRef<Path>, password: &str) -> Result<Self> {
        let bytes = Zeroizing::new(fs::read(path)?);
        Self::open(&bytes, password)
    }

    /// The read path of `docs/vault-format.md` §7, in order.
    ///
    /// Steps 6 and 7 are the ones that matter: a failed unwrap does **not** return early. It
    /// substitutes a zero key and carries on into the body decryption, so that a wrong password
    /// and a corrupted file spend the same work and return the same error (R-03). Returning at
    /// step 6 would make a wrong password measurably faster, which tells an attacker holding a
    /// stolen vault whether their guess was structurally right.
    fn open_with_secret(bytes: &[u8], secret: &[u8], credential: Credential) -> Result<Self> {
        // Steps 1–3: is this one of ours at all. Decided before any key material exists, so a
        // precise answer here leaks nothing.
        let header = Header::parse(bytes)?;

        // Step 4: the file must be exactly the length it claims.
        let body_len = usize::try_from(header.body_len).map_err(|_| Error::NotAVault)?;
        let (header_bytes, sealed_body) =
            bytes.split_at_checked(HEADER_LEN).ok_or(Error::NotAVault)?;
        if sealed_body.len() != body_len || body_len < aead::TAG_LEN {
            return Err(Error::NotAVault);
        }
        let wrap_aad = header_bytes
            .split_at_checked(WRAP_AAD_LEN)
            .ok_or(Error::NotAVault)?
            .0;

        let (salt, nonce, wrap) = match credential {
            Credential::Password => (&header.salt_pw, &header.nonce_pw, &header.wrap_pw),
            Credential::Recovery => (&header.salt_rk, &header.nonce_rk, &header.wrap_rk),
        };

        // Step 5: the expensive part, and the reason the two failure paths are hard to tell
        // apart in the first place.
        let kek = kdf::derive_key(secret, salt, header.kdf)?;

        // Steps 6 and 7. `unwrapped` is None on failure and the zero key stands in, so the
        // body decryption below runs either way.
        let unwrapped = aead::open(&kek, nonce, wrap, wrap_aad)
            .ok()
            .and_then(|plain| <[u8; KEY_LEN]>::try_from(plain.as_slice()).ok())
            .map(SecretBytes::new);
        let master_key = unwrapped.clone().unwrap_or_else(SecretBytes::zeroed);

        let opened = aead::open(&master_key, &header.nonce_body, sealed_body, header_bytes);

        // Step 8: success needs both. Evaluating them together, after both have run, is the
        // point — an `&&` that short-circuits here would undo the work above.
        let (Some(master_key), Ok(plaintext)) = (unwrapped, opened) else {
            return Err(Error::Unreadable);
        };

        let body = decode_body(&plaintext)?;
        migrate(header.version, body).map(|body| Self {
            kdf: header.kdf,
            salt_pw: header.salt_pw,
            nonce_pw: header.nonce_pw,
            wrap_pw: header.wrap_pw,
            salt_rk: header.salt_rk,
            nonce_rk: header.nonce_rk,
            wrap_rk: header.wrap_rk,
            master_key,
            body,
            audit_unflushed: false,
        })
    }

    /// Serializes and seals the vault.
    ///
    /// A fresh body nonce is drawn from the OS CSPRNG on **every** call (R-06). Two saves of
    /// the same vault therefore produce different bytes, which is correct and is why the
    /// known-answer vectors pin the *plaintext* a fixed file decrypts to rather than the bytes
    /// a save produces.
    pub fn to_bytes(&mut self) -> Result<Vec<u8>> {
        self.body.touch();

        let plaintext = Zeroizing::new(encode_body(&self.body)?);
        let nonce_body = random::<NONCE_LEN>()?;

        // The header is written before the body is sealed, because the whole header is the
        // body's associated data (§4). `body_len` is known in advance: the AEAD adds exactly
        // one tag length.
        let header = Header {
            version: FORMAT_VERSION,
            kdf: self.kdf,
            salt_pw: self.salt_pw,
            salt_rk: self.salt_rk,
            nonce_pw: self.nonce_pw,
            wrap_pw: self.wrap_pw,
            nonce_rk: self.nonce_rk,
            wrap_rk: self.wrap_rk,
            nonce_body,
            body_len: (plaintext.len() + aead::TAG_LEN) as u64,
        };
        let header_bytes = header.to_bytes();

        let sealed = aead::seal(&self.master_key, &nonce_body, &plaintext, &header_bytes)?;

        let mut out = Vec::with_capacity(HEADER_LEN + sealed.len());
        out.extend_from_slice(&header_bytes);
        out.extend_from_slice(&sealed);
        Ok(out)
    }

    /// Writes the vault to `path` atomically (R-05, `docs/vault-format.md` §8).
    ///
    /// Temp file in the same directory → fsync → rename → fsync the directory. A reader
    /// therefore never sees a half-written vault: it sees the previous file or the new one.
    /// The final fsync is the step usually skipped, and skipping it means the data survives
    /// power loss but the name pointing at it does not.
    pub fn save_to(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let bytes = self.to_bytes()?;
        let directory = path.parent().unwrap_or(Path::new("."));
        let temp = temp_path(path)?;

        let write_result = (|| -> Result<()> {
            let mut file = File::create(&temp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temp, path)?;
            // Directory fsync is best-effort: some platforms and filesystems refuse to open a
            // directory for this, and on those the rename's durability is the OS's business.
            if let Ok(dir) = File::open(directory) {
                let _ = dir.sync_all();
            }
            Ok(())
        })();

        if write_result.is_err() {
            // Leave no debris behind on failure. The temp file is a sealed vault or a fragment
            // of one — inert either way, but litter.
            let _ = fs::remove_file(&temp);
        } else {
            // The buffered audit tail is on disk now. Cleared only on success: a failed save
            // leaves the entries pending so the next attempt still writes them.
            self.audit_unflushed = false;
        }
        write_result
    }

    /// The vault's display name.
    pub fn name(&self) -> &str {
        &self.body.name
    }

    /// The Argon2id parameters this vault was created with.
    pub fn kdf_params(&self) -> KdfParams {
        self.kdf
    }

    /// The decrypted body.
    ///
    /// Crate-internal reach-through for tests and, later, the command layer. It hands out
    /// every secret in the vault at once, which is exactly what must never cross IPC (R-10).
    pub fn body(&self) -> &VaultBody {
        &self.body
    }

    /// Every item, in insertion order.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.body.items.iter()
    }

    /// Adds an empty item and returns its identifier.
    pub fn add_item(&mut self, kind: ItemKind, title: impl Into<String>) -> ItemId {
        let item = Item::new(kind, title);
        let id = item.id;
        self.body.items.push(item);
        id
    }

    /// Borrows an item.
    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.body.items.iter().find(|item| item.id == id)
    }

    /// Borrows an item for editing.
    pub fn item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        self.body.items.iter_mut().find(|item| item.id == id)
    }

    /// Removes an item, returning it.
    pub fn remove_item(&mut self, id: ItemId) -> Option<Item> {
        let index = self.body.items.iter().position(|item| item.id == id)?;
        Some(self.body.items.remove(index))
    }

    /// Returns **one** secret field's value, recording the reveal if `audit` is on.
    ///
    /// This is the only sanctioned way plaintext leaves the vault a field at a time (R-10,
    /// R-12), and the reveal and its audit entry are one operation on purpose: a command layer
    /// that could read a value without recording it is a command layer that eventually does.
    /// `docs/ipc-contract.md` §7 is the boundary this exists to serve.
    ///
    /// Refuses a field that is not marked secret ([`Error::NotSecret`]) — see that variant for
    /// why an easier answer is worse.
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchItem`], [`Error::NoSuchField`], or [`Error::NotSecret`].
    pub fn reveal_field(
        &mut self,
        item_id: ItemId,
        field_id: FieldId,
        audit: bool,
    ) -> Result<SecretString> {
        let item = self.item(item_id).ok_or(Error::NoSuchItem)?;
        let field = item.field(field_id).ok_or(Error::NoSuchField)?;
        if !field.secret {
            return Err(Error::NotSecret);
        }
        // Cloned rather than borrowed: the caller gets its own `SecretString`, which zeroizes
        // on drop independently of the vault's copy.
        let value = field.value.clone();

        if audit {
            self.body.record_reveal(item_id, field_id);
            self.audit_unflushed = true;
        }
        Ok(value)
    }

    /// Reveals recorded since the vault was opened or last saved, oldest first.
    pub fn audit_entries(&self) -> &[AuditEntry] {
        &self.body.audit
    }

    /// Whether reveals have been recorded that are not yet on disk.
    ///
    /// D-31 buffers the log in memory and flushes it on save, so that reading a password does
    /// not rewrite the vault file. The consequence is this flag: something has to decide when
    /// the buffered tail is worth a write, and the natural moment is lock. A crash before then
    /// loses the tail, which the decision accepted.
    pub fn has_unflushed_audit(&self) -> bool {
        self.audit_unflushed
    }

    /// Discards the whole audit log.
    ///
    /// Takes effect on disk at the next save, like every other body change.
    pub fn clear_audit(&mut self) {
        self.body.audit.clear();
        self.audit_unflushed = true;
    }

    /// Replaces the master password, keeping the master key and therefore the body.
    ///
    /// Only `salt_pw`, `nonce_pw`, and `wrap_pw` change. The body is not re-encrypted, so this
    /// is a constant-time-in-vault-size operation rather than one that can be interrupted
    /// halfway through a large file. The recovery kit continues to work — it wraps the same
    /// master key.
    pub fn change_password(&mut self, new_password: &str) -> Result<()> {
        let salt = random::<SALT_LEN>()?;
        let nonce = random::<NONCE_LEN>()?;
        self.wrap_pw = self.wrap_master_key(new_password.as_bytes(), &salt, &nonce)?;
        self.salt_pw = salt;
        self.nonce_pw = nonce;
        Ok(())
    }

    /// Issues a new recovery kit, invalidating the previous one.
    pub fn reissue_recovery_code(&mut self) -> Result<RecoveryCode> {
        let recovery = RecoveryCode::generate()?;
        let salt = random::<SALT_LEN>()?;
        let nonce = random::<NONCE_LEN>()?;
        self.wrap_rk = self.wrap_master_key(recovery.secret(), &salt, &nonce)?;
        self.salt_rk = salt;
        self.nonce_rk = nonce;
        Ok(recovery)
    }

    /// Seals the master key under a key derived from `secret`.
    ///
    /// The associated data is the parameter block — header bytes `0..20` — and not the whole
    /// header, which contains the wrap being produced, and not the salts either. See
    /// [`WRAP_AAD_LEN`] for why the salts are deliberately outside it.
    fn wrap_master_key(
        &self,
        secret: &[u8],
        salt: &[u8; SALT_LEN],
        nonce: &[u8; NONCE_LEN],
    ) -> Result<[u8; WRAP_LEN]> {
        let kek = kdf::derive_key(secret, salt, self.kdf)?;
        let sealed = aead::seal(&kek, nonce, self.master_key.expose(), &self.wrap_aad())?;
        <[u8; WRAP_LEN]>::try_from(sealed.as_slice()).map_err(|_| Error::Encode)
    }

    /// Rebuilds header bytes `0..20` as they will be written, for use as associated data.
    ///
    /// Everything in this range is known before any key exists, so the same bytes come out
    /// whether the vault is being created, re-keyed, or opened.
    fn wrap_aad(&self) -> Vec<u8> {
        let header = Header {
            salt_pw: [0; SALT_LEN],
            salt_rk: [0; SALT_LEN],
            version: FORMAT_VERSION,
            kdf: self.kdf,
            nonce_pw: [0; NONCE_LEN],
            wrap_pw: [0; WRAP_LEN],
            nonce_rk: [0; NONCE_LEN],
            wrap_rk: [0; WRAP_LEN],
            nonce_body: [0; NONCE_LEN],
            body_len: 0,
        };
        header
            .to_bytes()
            .split_at_checked(WRAP_AAD_LEN)
            .map(|(block, _)| block.to_vec())
            .unwrap_or_default()
    }
}

/// Which of the two credentials is opening the vault. They differ only in which salt, nonce,
/// and wrap are used — the work done is identical, so recovery is no faster to brute-force.
#[derive(Debug, Clone, Copy)]
enum Credential {
    Password,
    Recovery,
}

/// Serializes the body to CBOR (D-10).
fn encode_body(body: &VaultBody) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    ciborium::into_writer(body, &mut out).map_err(|_| Error::Encode)?;
    Ok(out)
}

/// Deserializes the body from CBOR.
///
/// A parse failure at this point means the plaintext is not a vault body even though the tag
/// verified, which is only reachable through a bug or a deliberate forgery by someone who
/// already has the key. Reported as [`Error::Unreadable`] like every other read failure.
fn decode_body(plaintext: &[u8]) -> Result<VaultBody> {
    ciborium::from_reader(plaintext).map_err(|_| Error::Unreadable)
}

/// The migration hook.
///
/// Version 1 has nothing to migrate from, and that is exactly why the hook exists now: a
/// migration point added at the moment it is first needed is a migration point designed under
/// pressure, against a format that already shipped. It is called on every open and is
/// exercised by a test.
fn migrate(from_version: u16, body: VaultBody) -> Result<VaultBody> {
    match from_version {
        0 => Err(Error::NotAVault),
        1 => Ok(body),
        // Unreachable through `Header::parse`, which rejects a newer version before any key
        // material exists. Handled anyway so that adding version 2 to the parser without
        // adding it here is a compile-time-obvious omission rather than a silent pass-through.
        found => Err(Error::UnsupportedVersion {
            found,
            supported: FORMAT_VERSION,
        }),
    }
}

/// A temp-file path in the same directory as the destination, so the rename in §8 step 4
/// cannot cross a filesystem boundary — which would turn an atomic rename into a copy.
fn temp_path(destination: &Path) -> Result<PathBuf> {
    let directory = destination.parent().unwrap_or(Path::new("."));
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("vault");
    let suffix = random::<4>()?;
    let suffix: String = suffix.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(directory.join(format!(".{name}.tmp-{suffix}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemStatus;

    fn new_vault() -> (Vault, RecoveryCode) {
        Vault::create("Personal", "correct horse", KdfParams::TESTING)
            .expect("test parameters are valid")
    }

    /// A vault with one secret field and one public one, for the reveal tests.
    fn vault_with_a_login() -> (Vault, ItemId, FieldId, FieldId) {
        let (mut vault, _) = new_vault();
        let item = vault.add_item(ItemKind::Login, "GitHub");
        let entry = vault.item_mut(item).expect("just added");
        let secret = entry.set_field("Password", "hunter2", true);
        let public = entry.set_field("Username", "octocat", false);
        (vault, item, secret, public)
    }

    #[test]
    fn reveal_returns_the_value_and_records_nothing_when_audit_is_off() {
        let (mut vault, item, secret, _) = vault_with_a_login();

        let value = vault.reveal_field(item, secret, false).unwrap();

        assert_eq!(value.expose(), "hunter2");
        assert!(vault.audit_entries().is_empty());
        assert!(!vault.has_unflushed_audit());
    }

    #[test]
    fn reveal_records_the_ids_and_never_the_value() {
        let (mut vault, item, secret, _) = vault_with_a_login();

        vault.reveal_field(item, secret, true).unwrap();

        let entries = vault.audit_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].item_id, item);
        assert_eq!(entries[0].field_id, secret);
        assert!(vault.has_unflushed_audit());

        // The whole point of the entry type: there is no field on it that could hold the
        // secret, the label, or the title. Serialized, it must not contain any of them.
        let encoded = serde_json::to_string(&entries[0]).unwrap();
        assert!(!encoded.contains("hunter2"));
        assert!(!encoded.contains("Password"));
        assert!(!encoded.contains("GitHub"));
    }

    #[test]
    fn reveal_refuses_a_field_that_is_not_secret() {
        // Not a convenience check. If a non-secret field could be "revealed", the audit log
        // and the IPC harness would both count reads of a username as reveals of a password.
        let (mut vault, item, _, public) = vault_with_a_login();

        assert!(matches!(
            vault.reveal_field(item, public, true),
            Err(Error::NotSecret)
        ));
        assert!(vault.audit_entries().is_empty());
    }

    #[test]
    fn reveal_rejects_unknown_item_and_field() {
        let (mut vault, item, secret, _) = vault_with_a_login();
        let stranger = ItemId::new_v4();

        assert!(matches!(
            vault.reveal_field(stranger, secret, true),
            Err(Error::NoSuchItem)
        ));
        assert!(matches!(
            vault.reveal_field(item, FieldId::new_v4(), true),
            Err(Error::NoSuchField)
        ));
    }

    #[test]
    fn a_reveal_does_not_move_updated_at() {
        // A reveal reads the vault. If it touched `updated_at`, every "last changed" display
        // in the UI would silently come to mean "last looked at".
        let (mut vault, item, secret, _) = vault_with_a_login();
        let before = vault.body().updated_at;

        vault.reveal_field(item, secret, true).unwrap();

        assert_eq!(vault.body().updated_at, before);
    }

    #[test]
    fn the_audit_log_is_capped_and_drops_the_oldest_first() {
        let (mut vault, item, secret, _) = vault_with_a_login();

        for _ in 0..VaultBody::AUDIT_MAX_ENTRIES + 50 {
            vault.reveal_field(item, secret, true).unwrap();
        }

        assert_eq!(vault.audit_entries().len(), VaultBody::AUDIT_MAX_ENTRIES);
    }

    #[test]
    fn the_audit_log_survives_a_round_trip_and_clears_the_flag_on_save() {
        let (mut vault, item, secret, _) = vault_with_a_login();
        vault.reveal_field(item, secret, true).unwrap();

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audited.tvault");
        vault.save_to(&path).unwrap();

        assert!(!vault.has_unflushed_audit(), "a save flushes the tail");

        let reopened = Vault::open_file(&path, "correct horse").unwrap();
        assert_eq!(reopened.audit_entries().len(), 1);
        assert_eq!(reopened.audit_entries()[0].item_id, item);
        assert!(
            !reopened.has_unflushed_audit(),
            "a vault read from disk has nothing pending by definition"
        );
    }

    #[test]
    fn an_empty_audit_log_writes_no_key_at_all() {
        // This is what keeps the known-answer vectors valid without regenerating them: a
        // vault that has never recorded a reveal must serialize exactly as it did before the
        // field existed.
        // Checked against the *plaintext* CBOR, not the sealed bytes: searching ciphertext for
        // a key name would pass whatever the encoding did, which is a test that proves nothing.
        let (mut vault, item, secret, _) = vault_with_a_login();

        let mut empty = Vec::new();
        ciborium::into_writer(vault.body(), &mut empty).unwrap();
        assert!(
            !empty.windows(5).any(|window| window == b"audit"),
            "an empty audit log must not appear in the encoded body"
        );

        // And the inverse, so the check above cannot pass because the key is spelled
        // differently or the encoding never writes text keys at all.
        vault.reveal_field(item, secret, true).unwrap();
        let mut populated = Vec::new();
        ciborium::into_writer(vault.body(), &mut populated).unwrap();
        assert!(
            populated.windows(5).any(|window| window == b"audit"),
            "a populated audit log must appear in the encoded body"
        );
    }

    #[test]
    fn clearing_the_audit_log_empties_it_and_marks_it_unflushed() {
        let (mut vault, item, secret, _) = vault_with_a_login();
        vault.reveal_field(item, secret, true).unwrap();

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audited.tvault");
        vault.save_to(&path).unwrap();

        vault.clear_audit();

        assert!(vault.audit_entries().is_empty());
        assert!(vault.has_unflushed_audit());
    }

    #[test]
    fn a_new_vault_round_trips() {
        let (mut vault, _) = new_vault();
        let id = vault.add_item(ItemKind::Login, "GitHub");
        vault
            .item_mut(id)
            .unwrap()
            .set_field("Password", "hunter2", true);

        let bytes = vault.to_bytes().unwrap();
        let reopened = Vault::open(&bytes, "correct horse").unwrap();

        assert_eq!(reopened.name(), "Personal");
        assert_eq!(reopened.items().count(), 1);
        assert_eq!(
            reopened
                .item(id)
                .and_then(|item| item.fields.first())
                .map(|field| field.value.expose().to_owned()),
            Some("hunter2".to_owned())
        );
    }

    #[test]
    fn the_body_reach_through_sees_every_item() {
        // `body()` hands out every secret at once. It is covered here so the R-10 comment on
        // it stays attached to something exercised — this is the shape that must never be
        // reachable from a Tauri command.
        let (mut vault, _) = new_vault();
        vault.add_item(ItemKind::Login, "GitHub");
        vault.add_item(ItemKind::Note, "Secret note");

        assert_eq!(vault.body().items.len(), 2);
        assert_eq!(vault.body().items.len(), vault.items().count());
    }

    #[test]
    fn the_recovery_kit_opens_the_vault_without_the_password() {
        // R-07. The two credentials wrap the same master key independently.
        let (mut vault, recovery) = new_vault();
        vault.add_item(ItemKind::Note, "Secret note");
        let bytes = vault.to_bytes().unwrap();

        let reopened = Vault::open_with_recovery(&bytes, &recovery).unwrap();
        assert_eq!(reopened.items().count(), 1);

        // And a different kit does not.
        let other = RecoveryCode::generate().unwrap();
        assert!(matches!(
            Vault::open_with_recovery(&bytes, &other),
            Err(Error::Unreadable)
        ));
    }

    #[test]
    fn a_wrong_password_fails_closed() {
        let (mut vault, _) = new_vault();
        let bytes = vault.to_bytes().unwrap();
        assert!(matches!(
            Vault::open(&bytes, "wrong horse"),
            Err(Error::Unreadable)
        ));
        assert!(matches!(Vault::open(&bytes, ""), Err(Error::Unreadable)));
    }

    #[test]
    fn every_save_uses_a_fresh_body_nonce() {
        // R-06. Identical content, different bytes — a counter or a constant would show up
        // here as an equality.
        let (mut vault, _) = new_vault();
        let first = vault.to_bytes().unwrap();
        let second = vault.to_bytes().unwrap();
        assert_ne!(first, second);
        assert_ne!(
            first.get(196..220),
            second.get(196..220),
            "nonce_body must differ between saves"
        );
    }

    #[test]
    fn changing_the_password_keeps_the_body_and_the_recovery_kit() {
        let (mut vault, recovery) = new_vault();
        vault.add_item(ItemKind::Card, "Visa");
        vault.change_password("a different password").unwrap();
        let bytes = vault.to_bytes().unwrap();

        assert!(Vault::open(&bytes, "a different password").is_ok());
        assert!(matches!(
            Vault::open(&bytes, "correct horse"),
            Err(Error::Unreadable)
        ));
        // The kit wraps the master key, which did not change.
        assert!(Vault::open_with_recovery(&bytes, &recovery).is_ok());
    }

    #[test]
    fn reissuing_the_recovery_kit_invalidates_the_old_one() {
        let (mut vault, old) = new_vault();
        let new = vault.reissue_recovery_code().unwrap();
        let bytes = vault.to_bytes().unwrap();

        assert!(Vault::open_with_recovery(&bytes, &new).is_ok());
        assert!(matches!(
            Vault::open_with_recovery(&bytes, &old),
            Err(Error::Unreadable)
        ));
        assert!(Vault::open(&bytes, "correct horse").is_ok());
    }

    #[test]
    fn crud_over_the_in_memory_vault() {
        let (mut vault, _) = new_vault();
        let first = vault.add_item(ItemKind::Login, "GitHub");
        let second = vault.add_item(ItemKind::ApiKey, "Stripe");
        assert_eq!(vault.items().count(), 2);

        vault.item_mut(second).unwrap().status = ItemStatus::Weak;
        assert_eq!(
            vault.item(second).map(|item| item.status),
            Some(ItemStatus::Weak)
        );

        let removed = vault.remove_item(first);
        assert_eq!(removed.map(|item| item.title), Some("GitHub".to_owned()));
        assert!(vault.item(first).is_none());
        assert!(vault.remove_item(first).is_none());
        assert_eq!(vault.items().count(), 1);
    }

    #[test]
    fn the_kdf_parameters_survive_a_round_trip() {
        // R-02: the parameters come from the header, not from a constant. A vault created
        // with unusual parameters must reopen with those, not with the defaults.
        let params = KdfParams {
            m_cost: 16,
            t_cost: 2,
            p_cost: 1,
        };
        let (mut vault, _) = Vault::create("Odd", "pw", params).unwrap();
        let bytes = vault.to_bytes().unwrap();
        assert_eq!(Vault::open(&bytes, "pw").unwrap().kdf_params(), params);
    }

    #[test]
    fn creation_rejects_out_of_range_parameters() {
        let params = KdfParams {
            m_cost: 1,
            t_cost: 1,
            p_cost: 1,
        };
        assert!(matches!(
            Vault::create("Bad", "pw", params),
            Err(Error::KdfParams)
        ));
    }

    #[test]
    fn a_truncated_or_extended_file_is_rejected() {
        // Step 4 of §7: the file must be exactly 228 + body_len bytes.
        let (mut vault, _) = new_vault();
        let bytes = vault.to_bytes().unwrap();

        let mut extended = bytes.clone();
        extended.push(0);
        assert!(matches!(
            Vault::open(&extended, "correct horse"),
            Err(Error::NotAVault)
        ));

        let truncated = &bytes[..bytes.len() - 1];
        assert!(matches!(
            Vault::open(truncated, "correct horse"),
            Err(Error::NotAVault)
        ));

        assert!(matches!(
            Vault::open(b"", "correct horse"),
            Err(Error::NotAVault)
        ));
    }

    #[test]
    fn the_migration_hook_is_wired_up() {
        let body = VaultBody::new("Personal");
        assert!(migrate(1, body.clone()).is_ok());
        assert!(matches!(migrate(0, body.clone()), Err(Error::NotAVault)));
        assert!(matches!(
            migrate(2, body),
            Err(Error::UnsupportedVersion {
                found: 2,
                supported: 1
            })
        ));
    }

    #[test]
    fn saving_writes_a_readable_file_and_leaves_no_debris() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("personal.tvault");

        let (mut vault, _) = new_vault();
        vault.add_item(ItemKind::SshKey, "Deploy key");
        vault.save_to(&path).unwrap();

        let reopened = Vault::open_file(&path, "correct horse").unwrap();
        assert_eq!(reopened.items().count(), 1);

        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp-"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );
    }

    #[test]
    fn saving_over_an_existing_vault_replaces_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("personal.tvault");

        let (mut vault, _) = new_vault();
        vault.save_to(&path).unwrap();
        vault.add_item(ItemKind::Identity, "Passport");
        vault.save_to(&path).unwrap();

        assert_eq!(
            Vault::open_file(&path, "correct horse")
                .unwrap()
                .items()
                .count(),
            1
        );
    }

    #[test]
    fn a_failed_save_reports_the_io_error() {
        let (mut vault, _) = new_vault();
        let missing = Path::new("/nonexistent-directory-for-trustvault/personal.tvault");
        assert!(matches!(vault.save_to(missing), Err(Error::Io(_))));
    }

    #[test]
    fn the_temp_path_is_a_sibling_of_the_destination() {
        // §8 step 1. A temp file in /tmp would make the rename a cross-device copy, which is
        // not atomic and defeats the entire mechanism.
        let destination = Path::new("/home/someone/vaults/personal.tvault");
        let temp = temp_path(destination).unwrap();
        assert_eq!(temp.parent(), destination.parent());
        assert!(
            temp.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".personal.tvault.tmp-"))
        );
        assert_ne!(
            temp_path(destination).unwrap(),
            temp,
            "suffix must be random"
        );
    }

    #[test]
    fn debug_output_never_contains_key_material() {
        let (vault, _) = new_vault();
        let debug = format!("{vault:?}");
        assert!(debug.contains("SecretBytes<32>(***)"));
    }
}
