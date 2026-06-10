# Changelog

All notable changes are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.2] — 2026-06-10

### Added
- CONTRIBUTING.md: setup, dev loop, commit conventions, release process
- Release CI workflow: builds NSIS + MSI installers on `vX.Y.Z` tags, creates draft GitHub Release
- Screenshots (dashboard + tray popup) in `.github/screenshots/`
- GitHub issue templates (bug report, feature request) and PR template with checklist
- Installer metadata: publisher, copyright, category, short/long description

### Changed
- README: `<div align="center">` layout, bold tagline, `flat-square` badges with logos (CI, release, license, Windows, Rust), feature table, `---` section separators, Install section first
- `.gitignore`: exclude `src-tauri/gen/schemas/` (auto-generated)

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
