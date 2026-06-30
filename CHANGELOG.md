# Changelog

All notable changes are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.3] — 2026-06-30

### Added
- Auto-update: the app checks GitHub Releases on startup and offers an in-app modal to download and install the new version (`tauri-plugin-updater` + `tauri-plugin-process`)
- Settings → "Buscar actualizaciones" button for an on-demand update check, separate from the automatic startup check
- Release CI now signs and publishes the `latest.json` updater manifest alongside the NSIS/MSI installers

### Fixed
- Toast notifications (port opened/closed, reserved port occupied) silently failed to show: `tauri-plugin-notification` only sets the Windows AppUserModelID when running the installed app (skipped for anything launched from `target/debug` or `target/release`) and swallows `show()` errors internally. Replaced with a direct `notify-rust` call that always sets the app id and logs failures instead of swallowing them.
- CLI: `killport` with no subcommand panicked (`args[2..]` on a 1-element argv) instead of listing ports
- `kill_port`/`kill_ports` IPC commands accepted any PID from the webview with no validation against a live scan
- `kill_tree` guarded the kill decision on a stale pre-kill snapshot name and skipped its PID-reuse check whenever a creation time was unreadable, instead of failing closed
- `is_alive` reported a process as dead when `OpenProcess` was merely access-denied, causing false "killed" results
- `guard::is_protected` matched only a bare process name, so a full path could in principle slip past the protected-process list
- `set_config` deduped `ignore_ports` without sorting first, so non-consecutive duplicates survived
- `restart_port` respawned even when the prior kill failed, risking a duplicate process on the same port
- UI: `unpkg.com` CDN script for Lucide icons conflicted with the `script-src 'self'` CSP (icons silently failed to load) and pulled unpinned third-party code into a privileged webview — vendored locally instead
- UI: stale PIDs lingered in the multi-select set after a process exited
- UI: `open_url`/`copy_url`/`open_folder` bypassed the error-handling wrapper used by the other actions
- UI: a config write could be clobbered by a refresh landing mid-flight
- UI: keyboard accessibility for custom button/toggle controls, ARIA labels, and a live region for toasts

### Changed
- `list_ports`, `kill_port`, `kill_ports`, `restart_port`, `check_firewall`, `get_qr_code` IPC commands moved off the main thread (`spawn_blocking`) so a slow scan or `netsh` call no longer freezes the UI
- `kill_port`/`kill_ports`/`restart_port` now surface kill/restart failures to the frontend instead of silently returning 0/false

## [0.1.2] — 2026-06-10

### Added
- CONTRIBUTING.md: setup, dev loop, commit conventions, release process
- Release CI workflow: builds NSIS + MSI installers on `vX.Y.Z` tags, creates draft GitHub Release
- Screenshots (dashboard + tray popup) in `.github/screenshots/`
- GitHub issue templates (bug report, feature request) and PR template with checklist
- Installer metadata: publisher, copyright, category, short/long description

### Fixed
- `cargo fmt` applied to 5 pre-existing files (`classify.rs`, `kill.rs`, `probe.rs`, `ports.rs`, `commands.rs`) — CI was failing on format check

### Changed
- README: `<div align="center">` layout, bold tagline, `flat-square` badges with logos (CI, release, license, Windows, Rust), feature table, `---` section separators, Install section first
- `.gitignore`: exclude `src-tauri/gen/schemas/` (auto-generated)
- CI + Release workflows: opt into Node.js 24 runner via `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`

## [0.1.1] — 2025-06-10

### Added
- System tray with popup port list and open/close notifications
- Main dashboard window with port inspector, protocol probe, and QR code generator
- Firewall rule check per port via `netsh`
- Autostart on login (Windows registry)
- Reserved port tracking with dedicated notifications
- Config persistence (TOML, `%APPDATA%\Killport\config.toml`)
- MIT license
- CI workflow (fmt + clippy -D warnings + tests on windows-latest)
- Release workflow (NSIS + MSI on version tags)

### Fixed
- MySQL protocol probe: panic on short server greeting (OOB slice)
- CLI `restart`: kill failure no longer silently continues to respawn
- Poll thread: shutdown flag wired to `quit_app` to enable clean exit
- `ports.rs`: `dwNumEntries` clamped to buffer capacity before unsafe slice construction
- `kill.rs` test: child process now waited after kill (clippy must-use)

### Changed
- Poll interval clamped to `[1, 300]` seconds; previously unbounded
- Tauri startup failure replaced `.expect()` with graceful `exit(1)` + message
- Internal modules marked `pub(crate)` — only `autostart`, `config`, `ports` stay public
- `anyhow` removed from CLI (unused dependency)
- `project::Project.marker` field removed (dead code)
- Tauri installer: publisher, copyright, category, Start Menu shortcut, per-user install mode

## [0.1.0] — 2025-06-05

Initial bootstrap: workspace scaffold, core library, CLI, and Tauri shell.
