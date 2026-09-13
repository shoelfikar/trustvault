# Release process

TrustVault releases are built only by `.github/workflows/release.yml` from an annotated `vX.Y.Z`
tag. The current workflow derives every package version from the tag, builds the Linux x86-64
`.deb` and `.AppImage`, tests both in clean containers, checks the packaged binary's `--version`
output, generates SHA-256 checksums, and only then creates the GitHub release.

macOS arm64 and Windows x86-64 are planned but deferred under D-97. Their credential contract and
verification procedure remain below so resuming them does not require redesigning the release.

## Current Linux release

Configure the `release` GitHub environment with required reviewers. The Linux-only workflow uses
no external signing secrets.

## Deferred macOS and Windows credentials

When those platforms resume, add these secrets to the protected `release` environment:

- `APPLE_CERTIFICATE`: base64 of the Developer ID Application `.p12` export.
- `APPLE_CERTIFICATE_PASSWORD`: password used for that export.
- `APPLE_SIGNING_IDENTITY`: full `Developer ID Application: … (TEAMID)` identity.
- `APPLE_ID`: Apple Developer account email.
- `APPLE_PASSWORD`: an app-specific Apple password.
- `APPLE_TEAM_ID`: the ten-character team identifier.
- `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, and `AZURE_SUBSCRIPTION_ID`: an Entra application with a
  federated GitHub credential, not a stored client secret.
- `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, and `AZURE_CERTIFICATE_PROFILE`: the Azure
  Artifact Signing endpoint, account, and public-trust certificate profile.

The Azure principal gets only `Artifact Signing Certificate Profile Signer` on the selected
profile. GitHub's job receives `id-token: write` only for OIDC login; no Azure password is stored.

## Apple certificate handling

Create a Developer ID Application certificate in Apple Developer, install it in Keychain Access,
export the identity and private key to a password-protected `.p12`, and encode it with:

```sh
openssl base64 -A -in Developer-ID-Application.p12 -out apple-certificate.base64
```

Paste the encoded content into `APPLE_CERTIFICATE`, then immediately remove both local files and
empty Trash. Do not copy either file into this repository. The workflow imports the secret into an
ephemeral runner keychain; GitHub destroys the runner after the job. Rotate the certificate at once
if a `.p12`, its base64 representation, or its password ever enters git history or job output.

## Cutting a release

1. Update `CHANGELOG.md`, merge to `main`, and confirm CI is green.
2. Create and push an annotated tag: `git tag -a vX.Y.Z -m "TrustVault vX.Y.Z"`.
3. Approve the protected `release` environment run.
4. Check that the Linux build, both smoke tests, and the checksum job passed.
5. Download one asset and verify it against `SHA256SUMS` as described in `docs/install.md`.

Never re-point or reuse a release tag. A failed run publishes nothing; fix the cause and cut a new
patch tag. The signing identities can sign code, so their access is reviewed separately from normal
repository write access.

## Verification performed by current CI

- Linux: clean Ubuntu and Fedora containers install/run the packages and check the packaged
  executable's version.
- Release: exactly one `.deb` and one `.AppImage` must exist; SHA-256 checksums are generated from
  those exact files and uploaded beside them.

## Deferred verification to restore with those platforms

- macOS: `codesign --verify`, `spctl --assess`, and `xcrun stapler validate` before mounting the
  DMG and checking the copied app binary's version.
- Windows: Azure Artifact Signing followed by `Get-AuthenticodeSignature`, silent MSI install, and
  the installed executable's version.

The workflow intentionally has no updater. Adding one would create a new signed delivery channel
into a process that holds decrypted secrets and requires a separate threat model.
