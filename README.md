# Killport

[![CI](https://github.com/episuarez/killport/actions/workflows/ci.yml/badge.svg)](https://github.com/episuarez/killport/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Windows only](https://img.shields.io/badge/platform-Windows-0078d4)](https://github.com/episuarez/killport/releases)

Windows developer tool for managing TCP port processes. See what's listening on a port, kill it, or restart it — with enough context to know exactly what you're hitting.

## Features

- **Port scan** — all listening TCP ports, IPv4 + IPv6, via Win32 IP Helper API (no `netstat` parsing)
- **Process context** — runtime (Node, Python, PHP, Go, PostgreSQL, Redis, Docker…), detected framework (Vite, Next.js, Django, Laravel…), project name, parent process
- **Docker-aware** — maps container ports to their image and name
- **Windows services** — identifies SCM-registered services
- **Kill** — graceful (WM_CLOSE → wait → force) or immediate; kills entire process trees
- **Restart** — kills and respawns with the original command line and working directory
- **System tray** — lives in the tray; popup list + notifications on port open/close
- **Desktop dashboard** — full view with protocol probe (HTTP/WS/Redis/MySQL/gRPC…) and QR code for mobile
- **CLI** — headless `killport` binary for scripting

## Install

Download the latest `.exe` installer or `.msi` package from the [Releases](https://github.com/episuarez/killport/releases) page.

Requires **Windows 10 version 1803** or later (WebView2 runtime, ships with Windows since 2018).

## CLI usage

```
killport list                  # show dev ports
killport list --all            # include system processes
killport kill <port>           # kill gracefully
killport kill <port> --force   # kill immediately
killport kill --pid <pid>      # kill by PID
killport restart <port>        # kill + respawn
```

## Build from source

```bat
# Prerequisites: Rust stable, cargo-tauri
rustup update stable
cargo install tauri-cli

git clone https://github.com/episuarez/killport
cd killport

# Desktop app
cargo tauri build

# CLI only
cargo build -p killport-cli --release
```

Binaries end up in `target/release/`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
