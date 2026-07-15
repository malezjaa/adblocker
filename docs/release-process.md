# Windows release process

1. Configure a GitHub ruleset for `v*` tags that permits only release maintainers to create or update tags. Do not retarget a published release tag.
2. Set every Rust workspace package version to the release version, commit it, and create the matching `vX.Y.Z` tag.
3. Complete the manual Hardware Dev Center signing step for the pinned WinDivert x64 runtime. Commit the matching `WinDivert.dll`, `WinDivert.lib`, `WinDivert64.sys`, upstream `LICENSE.txt`, and their lowercase SHA-256 entries under `third_party/windivert/x64`. Cargo links the client against this exact import library.
4. Push the tag. The workflow runs quality checks, creates the portable x64 ZIP, verifies the driver signature, publishes SHA-256 checksums, creates GitHub provenance attestations, and then publishes the GitHub Release.

To rerun a release, use the workflow-dispatch input with an existing tag. Do not use it to manufacture a new version.

Consumers can verify the published provenance with GitHub CLI:

```powershell
gh attestation verify "vox-X.Y.Z-x86_64-pc-windows-msvc.zip" --repo malezjaa/adblocker
```
