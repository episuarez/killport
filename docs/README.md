# KillPort

Windows developer tool for managing TCP port processes. Lists what's listening, kills it, or restarts it — with enough context to know what you're hitting.

## What it does

- Scans all listening TCP ports (IPv4 + IPv6)
- Classifies processes by runtime: Node, Python, PHP, PostgreSQL, Redis, Docker, WSL, etc.
- Detects web frameworks: Vite, Next.js, Django, Laravel, Express, Angular, and more
- Identifies Docker containers, Windows services, and parent processes
- Kills gracefully (WM_CLOSE → wait → force) or immediately
- Kills entire process trees (npm → node, etc.)
- Restarts processes with their original command and working directory preserved

## Interfaces

| Interface | Description |
|-----------|-------------|
| **Desktop GUI** | Tauri app with system tray popup + main dashboard window |
| **CLI** | Headless `killport` binary for scripting and automation |

## Codebase layout

```
KillPort/
├── crates/
│   ├── killport-core/     # Shared logic: scanning, killing, classification
│   └── killport-cli/      # Headless CLI binary
├── src-tauri/             # Tauri backend: IPC commands, notifications, tray
├── ui/                    # Frontend: HTML/CSS/JS (no framework)
└── docs/                  # This directory
```

## Documentation

- [Architecture](./architecture/README.md)
- **Features**
  - [Port Scanning](./features/port-scanning/README.md)
  - [Process Killing](./features/process-killing/README.md)
  - [Process Restart](./features/process-restart/README.md)
  - [Process Classification](./features/process-classification/README.md)
  - [Framework Detection](./features/framework-detection/README.md)
  - [Docker Integration](./features/docker-integration/README.md)
  - [Windows Services](./features/windows-services/README.md)
  - [System Guard](./features/system-guard/README.md)
  - [Autostart](./features/autostart/README.md)
  - [Notifications](./features/notifications/README.md)
- **Interfaces**
  - [Desktop UI](./interfaces/desktop-ui/README.md)
  - [CLI](./interfaces/cli/README.md)
- [Configuration](./config/README.md)

## Tech stack

- **Rust 2021** — workspace monorepo
- **Tauri 2** — desktop shell
- **Windows Win32 / WinRT** — port enumeration, process control, services, registry, notifications
- **Vanilla HTML/CSS/JS** — frontend (no framework)
- **TOML** — user configuration
