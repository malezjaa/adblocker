# Signed WinDivert release inputs

Before publishing any release, place the x64 runtime files here:

- `WinDivert.dll`
- `WinDivert.lib` matching the DLL; Cargo links against this import library
- `WinDivert64.sys`
- `LICENSE.txt` containing the applicable upstream license text
- `SHA256SUMS.txt` with lowercase SHA-256 entries in `hash *filename` format

Cargo is configured to use this directory through `WINDIVERT_PATH`. The release workflow verifies all three binary hashes and the driver's Authenticode signature before compiling anything. These inputs are deliberately not downloaded during a release build.
