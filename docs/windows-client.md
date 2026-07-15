# Windows client

The optional Windows client redirects direct outbound UDP/53 traffic from applications that bypass the Windows DNS
resolver. It does not replace the system DNS configuration and does not currently intercept TCP DNS traffic.

Most installations only need the daemon and `cli.exe dns set`. Install the client only when a direct DNS bypass needs to
be intercepted.

## Configure the client

Before installation, create `%ProgramData%\Vox\client\config.toml`:

```toml
dns_server = "192.0.2.10:53"
# Optional: forward intercepted DNS queries to Vox over DoH instead of UDP.
doh = "192.0.2.10:443"
```

Replace the example addresses with the Vox daemon's reachable addresses. `dns_server` is required and is used as the
plain DNS upstream when `doh` is omitted. When `doh` is present, the client sends intercepted queries as HTTPS POST
requests to `https://doh.local:<doh-port>/dns-query`, connects that name directly to the configured address, and does
not use plain DNS for forwarding.

The DoH certificate must be trusted by the client machine and valid for `doh.local`. With Vox's default
self-signed strategy, install `%ProgramData%\Vox\daemon\certs\self_signed\ca.pem` as a trusted root CA on the client
before starting the service.

## Install and manage the service

Run these commands from an admin PowerShell prompt in the release directory:

```powershell
.\cli.exe service install client
.\cli.exe service status client
```

Use `start`, `stop`, `restart`, or `uninstall` to manage the `VoxWindowsClient` service.
