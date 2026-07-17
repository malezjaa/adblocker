<p align="center">
  <h1 align="center">Vox</h1>
  <p align="center">
    A private, self-hosted DNS filtering server for Windows.
    <br />
    Block ads and trackers, manage devices, and see what your network is asking for.
    <br /><br />
    <a href="#getting-started">Get started</a>
    ·
    <a href="docs/installation.md">Installation</a>
    ·
    <a href="docs/certificates.md">Certificates</a>
    ·
    <a href="#development">Development</a>
  </p>
</p>

<p align="center">
  <a href="https://github.com/malezjaa/adblocker/actions/workflows/windows-release.yml"><img src="https://github.com/malezjaa/adblocker/actions/workflows/windows-release.yml/badge.svg?branch=main" alt="Windows release workflow" /></a>
  <a href="https://github.com/malezjaa/adblocker/releases"><img src="https://img.shields.io/github/v/release/malezjaa/adblocker?display_name=tag&label=release" alt="Latest release" /></a>
  <a href="https://github.com/malezjaa/adblocker"><img src="https://img.shields.io/badge/platform-Windows%2010%20%2F%2011-0078D4" alt="Windows 10 and 11" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/built%20with-Rust-dea584" alt="Built with Rust" /></a>
</p>

> [!WARNING]
> Vox currently supports **Windows 10 and Windows 11 x64 only**. Installing the DNS server or Windows client requires an
> elevated PowerShell session.

## Why Vox?

Vox is a local DNS server and Windows client for people who want a practical layer of network-wide privacy without
giving up control. It keeps administration on your machine and provides the essentials in a focused package:

- **DNS filtering**: Choose from more than 20 popular blocklists.
- **DNS-over-HTTPS** with ACME, self-signed, or manually supplied certificates
- **Local dashboard** for live activity, query logs, lists, devices, rules, and DNS rewrites
- **Custom allow/block rules** and conditional DNS rewrites
- **Windows service integration** and a companion client that transparently redirects DNS

## Getting started

Download the latest Windows x64 archive from [Releases](https://github.com/malezjaa/adblocker/releases), then follow
the [installation guide](docs/installation.md). Vox defaults to self-signed DNS-over-HTTPS (DoH), so install OpenSSL
on the system path before the first daemon start.

```powershell
.\cli.exe service install daemon
.\cli.exe admin create
.\cli.exe dns set "Office PC"
```

The dashboard is available locally at [http://127.0.0.64](http://127.0.0.64). See
the [certificate guide](docs/certificates.md)
to use ACME, a manually supplied certificate, or TLS termination by a reverse proxy.

## Documentation

- [Installation and operations](docs/installation.md)
- [Certificate strategies](docs/certificates.md)
- [Windows client](docs/windows-client.md)
- [Configuration reference](docs/configuration.md)

## Development

Vox is a Rust workspace with a React/Vite dashboard. You will need Rust, [Node.js](https://nodejs.org/),
and [pnpm](https://pnpm.io/).

Vox also uses `openssl` crate with `vendored` feature which requires `perl` and `make` to be installed.

```powershell
git clone https://github.com/malezjaa/adblocker.git
cd adblocker
cargo test --workspace

cd dashboard
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
```

## Contributing

Issues and pull requests are welcome. Before opening a pull request, please run the checks
in [Development](#development), keep changes focused, and include tests when behavior changes.

## Third-party notices

The Windows client bundle includes the WinDivert driver. Its license and third-party notices are included in every
release archive.
