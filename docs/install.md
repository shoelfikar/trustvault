# Install TrustVault

The current release supports Linux x86-64. Download its artifact and `SHA256SUMS` from the same
GitHub release. macOS arm64 and Windows x86-64 packages are planned but deferred.

## Verify the download

On Linux or macOS:

```sh
sha256sum -c SHA256SUMS --ignore-missing
```

On Windows PowerShell, compare the output with the matching line in `SHA256SUMS`:

```powershell
Get-FileHash .\TrustVault_*.msi -Algorithm SHA256
```

Stop if the digest differs. A checksum detects a damaged or substituted download only when the
checksum file came from the intended GitHub release.

## Linux x86-64

Ubuntu and Debian users should download the `.deb` and run:

```sh
sudo apt install ./TrustVault_*.deb
```

For other distributions, download the `.AppImage`, make it executable, and start it:

```sh
chmod +x TrustVault_*.AppImage
./TrustVault_*.AppImage
```

Some distributions require FUSE 2 compatibility for AppImage. If the desktop blocks execution,
allow the file to run in its Properties dialog; do not disable system-wide application security.

## macOS arm64 — planned, not currently released

Download the `.dmg`, open it, and drag TrustVault to Applications. The release is signed with a
Developer ID Application identity, notarized by Apple, and has the notarization ticket stapled.

A new signing identity can still produce a Gatekeeper confirmation. Confirm that the dialog names
TrustVault and the expected developer. If macOS blocks it, open **System Settings → Privacy &
Security**, review the blocked-app message, and choose **Open Anyway** only after verifying the
checksum. Never use `xattr -dr com.apple.quarantine` or disable Gatekeeper globally.

Intel Macs are not supported by the first release.

## Windows x86-64 — planned, not currently released

Download and open the `.msi`. The installer is signed through Azure Artifact Signing. Windows
SmartScreen reputation accrues over time, so early releases can still show a warning even with a
valid signature. Select **More info**, verify that the publisher matches the TrustVault release
notes, and choose **Run anyway** only after checking the SHA-256 digest. Do not turn SmartScreen off.

Windows on Arm and 32-bit Windows are not supported.

## Confirm the installed version

The binary reports its release version without opening a vault:

```sh
trustvault --version
```

The output must match the release tag (without the leading `v`).
