# Vox for Windows x64

This package supports Windows 10 and Windows 11 x64 only. Extract it, open an elevated PowerShell prompt in the extracted directory, and use the management CLI to install the services.

```powershell
.\cli.exe service install daemon
.\cli.exe service install client --dns-server 192.0.2.10:53
```

Both services run as LocalSystem and are installed under `%ProgramFiles%\Vox`. Configuration, database files, downloaded data, and logs are stored under `%ProgramData%\Vox`.

Use `cli.exe service status daemon`, `start`, `stop`, and `uninstall`. Add `--purge-data` to uninstall only when you intentionally want to remove all Vox machine data.

The package includes `SHA256SUMS.txt`; verify it before installation. GitHub CLI can verify the published provenance with `gh attestation verify <archive> --repo malezjaa/adblocker`. The WinDivert driver requires administrator privileges and may be identified by endpoint protection because it intercepts DNS traffic.
