# Killport

<div align="center">

<img src=".github/screenshots/dashboard.png" alt="Kill Port dashboard" width="700">
&nbsp;&nbsp;
<img src=".github/screenshots/tray.png" alt="System tray popup" width="196">

### Kill the process on any port — and know exactly what you're hitting.

No more `netstat -ano | findstr :3000` followed by guessing what PID 18244 even is.
Killport tells you it's `next-app` running via `npm run dev`, and lets you kill or
restart it in one click — or one command.

[![CI](https://img.shields.io/github/actions/workflow/status/episuarez/killport/ci.yml?style=flat-square&label=CI)](https://github.com/episuarez/killport/actions)
[![Release](https://img.shields.io/github/v/release/episuarez/killport?style=flat-square&label=release)](https://github.com/episuarez/killport/releases)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square&logo=windows&logoColor=white)](https://github.com/episuarez/killport/releases)
[![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
<br>
[![Downloads](https://img.shields.io/github/downloads/episuarez/killport/total?style=flat-square&label=downloads&color=58A6FF)](https://github.com/episuarez/killport/releases)
[![Last commit](https://img.shields.io/github/last-commit/episuarez/killport?style=flat-square&color=7EE787)](https://github.com/episuarez/killport/commits/main)
[![Issues](https://img.shields.io/github/issues/episuarez/killport?style=flat-square&color=D2A8FF)](https://github.com/episuarez/killport/issues)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-FF7B72?style=flat-square)](CONTRIBUTING.md)

</div>

---

## Install

Download the latest `.exe` installer or `.msi` package from the [Releases](https://github.com/episuarez/killport/releases) page.

Requires **Windows 10 version 1803** or later (WebView2 runtime, included with Windows since 2018).

---

## Why Killport

Every dev day starts the same way: `EADDRINUSE`, a frozen dev server from yesterday,
a Docker container that never shut down. The fix is always the same three steps —
find the PID, figure out what it actually is so you don't kill the wrong thing,
then kill it — and Windows makes every one of those steps painful.

Killport collapses all three into one:

- **No more `netstat` + `tasklist` + `taskkill` combos.** One scan shows every
  listening port with the process, app, framework, and project behind it.
- **No more guessing if it's safe to kill.** A hard-coded guard list refuses to
  touch `lsass.exe`, `svchost.exe`, `services.exe` and friends — even by accident,
  even with `--force`.
- **No more stale ports after a kill.** `kill_tree` walks the whole process tree
  (`npm` → `node`, `docker-compose` → containers) so the port is actually freed,
  not just the parent.
- **No more retyping the start command.** Restart respawns the exact original
  command line in the original working directory.

---

## See it in action

<div align="center">
<img src=".github/screenshots/terminal-demo.svg" alt="Killport CLI demo: list ports, identify the process, kill it" width="560">
</div>

```
$ killport list
PORT   PID    KIND   APP        ORIGIN
3000   18244  node   next-app   npm run dev
5432   9001   pgsql  postgres   service: postgresql-x64-16

$ killport kill 3000
killed 1 process(es) in tree of pid 18244
```

<div align="center">
<img src=".github/screenshots/flow-demo.svg" alt="Scan, identify, kill or restart, port freed" width="640">
</div>

---

## What it does

| Feature | Details |
|---|---|
| **Port scan** | All listening TCP ports, IPv4 + IPv6, via Win32 IP Helper API — no `netstat` |
| **Process context** | Runtime (Node, Python, PHP, Go, PostgreSQL, Redis, Docker…) + detected framework (Vite, Next.js, Django, Laravel…) + project name |
| **Docker-aware** | Maps container ports to their image and name |
| **Windows services** | Identifies SCM-registered services |
| **System guard** | Hard-coded protected-process list — critical OS processes can never be killed, even by mistake |
| **Kill** | Graceful (WM\_CLOSE → wait → force) or immediate; kills entire process trees so the port is actually freed |
| **Restart** | Kills and respawns with the original command line and working directory |
| **System tray** | Popup port list, notifications on port open/close, reserved-port alerts |
| **Dashboard** | Protocol probe (HTTP/WS/Redis/MySQL/gRPC…), firewall rule check, QR code for mobile access |
| **CLI** | Headless `killport` binary for scripting and automation |

---

## CLI usage

```
killport list                  # dev ports only
killport list --all            # include system processes
killport kill <port>           # graceful kill
killport kill <port> --force   # immediate kill
killport kill --pid <pid>      # kill by PID
killport restart <port>        # kill + respawn
```

---

## Build from source

```bat
rustup update stable
cargo install tauri-cli

git clone https://github.com/episuarez/killport
cd killport

cargo tauri build              # desktop app + installer
cargo build -p killport-cli --release   # CLI only
```

Binaries land in `target/release/`.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, commit conventions, and the release process.

Found a process Killport refuses to kill that it shouldn't, or one it kills that it
shouldn't? Open an issue — the system guard list is the one place we'd rather be
told we're wrong.

---

## License

MIT — see [LICENSE](LICENSE).
