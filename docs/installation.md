# Installation and operations

Vox supports Windows 10 and Windows 11 x64. Installing its services requires an admin PowerShell session.

> [!WARNING]
> The optional Windows client uses the WinDivert driver to intercept DNS traffic.

## Before you begin

Download and extract the latest Windows x64 archive from [Releases](https://github.com/malezjaa/adblocker/releases).
Open **PowerShell as Administrator** in the extracted directory.

New installations use self-signed DoH by default. Install OpenSSL and make it available on the **system** `PATH` before
starting the daemon for the first time. See [Certificate strategies](certificates.md) for alternatives.

Each archive includes `SHA256SUMS.txt`. You can verify it before installation. GitHub CLI can also verify build
provenance:

```powershell
gh attestation verify <archive> --repo malezjaa/adblocker
```

## Install the daemon

```powershell
.\cli.exe service install daemon
.\cli.exe service status daemon
```

Installation copies the required files to `%ProgramFiles%\Vox`, creates and starts the `VoxDaemon` service, and creates
`%ProgramData%\Vox\daemon\config.toml` if it does not already exist.

Refer to [Configuration reference](./configuration.md) for more information.

## Create a dashboard account

```powershell
.\cli.exe admin create
```

Follow the interactive prompt, then sign in at [http://127.0.0.64](http://127.0.0.64). You can disable the dashboard
with the `dashboard` setting. CLI and configuration file are enough to manage the daemon.

## Configure the host's DNS adapter

Use a device name to keep dashboard analytics:

```powershell
.\cli.exe devices create --name "Office PC" --device-type windows
.\cli.exe dns set "Office PC"
.\cli.exe dns set
```

`dns set` configures the active adapter to use Vox and enables DoH. Use `--no-doh` only when unencrypted DNS is
intentional.

## Service management

```powershell
.\cli.exe service restart daemon
.\cli.exe service stop daemon
.\cli.exe service uninstall daemon
```

Add `--purge-data` to `uninstall` only when you intentionally want to remove all Vox data from the machine.

| Location                   | Purpose                                                              |
|----------------------------|----------------------------------------------------------------------|
| `%ProgramFiles%\Vox`       | Installed service binaries and WinDivert files                       |
| `%ProgramData%\Vox\daemon` | Server configuration, database, certificates, lists, and daemon data |
| `%ProgramData%\Vox\client` | Windows client configuration                                         |
| `%ProgramData%\Vox\logs`   | Runtime logs                                                         |

For certificate changes, configuration details, and restart requirements,
see [Configuration reference](configuration.md).
