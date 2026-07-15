# Configuration reference

The daemon configuration file is `%ProgramData%\Vox\daemon\config.toml`.

New installations create a configuration equivalent to the following defaults:

```toml
blocklists = ["oisd-big"]
dashboard = true

[dns]
enabled = true
port = 53

[doh]
enabled = true
port = 443

[resolver]
dnssec = false

[certs]
strategy = "self-signed"

[firewall]
open_ports = false
```

Cloudflare (`1.1.1.1` and `1.0.0.1`) is the default upstream resolver set.

## Applying changes

Vox watches its configuration file. Blocklists, rules, and upstream resolver settings are applied while the service is
running. Listener settings (`dns` and `doh`), certificate settings, firewall settings, and dashboard enablement
require a restart.

```powershell
.\cli.exe service restart daemon
```

Restarting after any manual configuration change is the safest option.

## Managing configuration

Use the dashboard for lists, rules, DNS rewrites, devices, and settings. The management CLI also supports:

```powershell
.\cli.exe devices list
.\cli.exe devices create --name "Office PC" --device-type windows
.\cli.exe dns set "Office PC"
```

Run `.\cli.exe --help` to see all commands and `.\cli.exe service --help` for service-management options. See
[Certificate strategies](certificates.md) for all certificate-related settings.
