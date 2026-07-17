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
doh = "doh.example.com"
```

Replace the example addresses with the Vox daemon's reachable addresses. `dns_server` is required and is used as the
plain DNS upstream when `doh` is omitted. When `doh` is present, the client sends intercepted queries as HTTPS POST
requests to `https://<doh-hostname>/dns-query` and does not use plain DNS for forwarding. A port can be included, for
example `doh = "doh.example.com:8443"`. An explicit HTTPS endpoint, such as
`https://doh.example.com/custom-dns-query`, is also supported.

The DoH certificate must be trusted by the client machine and valid for the configured DoH hostname. With Vox's
default-self-signed strategy, install `%ProgramData%\Vox\daemon\certs\self_signed\ca.pem` as a trusted root CA on the
client before starting the service.

An `IP:port` DoH configuration is also supported. It uses the `doh.local` endpoint and therefore requires a
certificate valid for `doh.local`.

## Install and manage the service

Run these commands from an admin PowerShell prompt in the release directory:

```powershell
.\cli.exe service install client
.\cli.exe service status client
```

Use `start`, `stop`, `restart`, or `uninstall` to manage the `VoxWindowsClient` service.
