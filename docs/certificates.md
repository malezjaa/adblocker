# Certificate strategies

Vox uses certificates for DNS-over-HTTPS (DoH). Choose one strategy in `%ProgramData%\Vox\daemon\config.toml`.
Certificate changes require a daemon restart:

```powershell
.\cli.exe service restart daemon
```

For a new ACME setup, complete the challenge before restarting the daemon, as described below.

| Need                                               | Strategy                |
|----------------------------------------------------|-------------------------|
| Private DoH on the local machine or LAN            | `self-signed` (default) |
| Publicly trusted certificate for a public hostname | `acme`                  |
| Existing certificate and private key               | `manual`                |
| TLS terminated by a reverse proxy or tunnel        | `none`                  |

## Self-signed (default)

```toml
[certs]
strategy = "self-signed"
```

On first startup, Vox uses OpenSSL to create a certificate authority (CA) and a `doh.local` server certificate. The CA
is added to the Windows certificate store on the daemon host. OpenSSL must be available on the system `PATH` used by
the service.

For other devices, install and trust `%ProgramData%\Vox\daemon\certs\self_signed\ca.pem` as a root CA. Firefox uses
its own certificate store. Either import this CA and trust it to identify websites, or enable
`security.enterprise_roots.enabled` in `about:config`.

To accept clients from outside the host, open the configured DNS and DoH ports:

```toml
[firewall]
open_ports = true
```

Restart the daemon after changing this setting.

## ACME

```toml
[certs]
strategy = "acme"

[certs.acme]
domain = "dns.example.com"
email = "admin@example.com"
challenge = "dns-01"
```

Use ACME only when Vox is reachable through a hostname you control, and you want a publicly trusted certificate. DNS-01
is the supported challenge type.

After saving the configuration, run the challenge from an elevated PowerShell prompt:

```powershell
.\cli.exe acme challenge
```

The CLI creates or reuses the ACME account stored in `%ProgramData%\Vox\daemon\acme-accounts.toml`, then prints the
DNS TXT record required by the certificate authority. Add the record at your DNS provider and press Return. If the DNS
record has not updated yet, wait and press Return again after waiting a moment.

After the CLI fetches the certificate, restart the daemon:

```powershell
.\cli.exe service restart daemon
```

When a certificate is nearing renewal, the daemon logs a warning. Run `cli.exe acme challenge` again to renew it. The
command exits without placing an order when the current certificate is still valid.

## Manual certificate

```toml
[certs]
strategy = "manual"

[certs.manual]
cert_path = 'C:\certificates\fullchain.pem'
key_path = 'C:\certificates\private-key.pem'
```

The certificate path must contain a PEM certificate chain and the key path must contain the matching PEM private key.
The `VoxDaemon` service runs as `LocalSystem`, so that account must be able to read both files. Restart the daemon after
replacing either file.

## TLS termination elsewhere

```toml
[certs]
strategy = "none"
```

Use this when a reverse proxy or tunnel, such as Caddy or Cloudflare Tunnel, terminates TLS. With this setting Vox
serves the DoH endpoint without TLS, so do not expose it directly to an untrusted network.
