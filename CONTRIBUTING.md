# Contributing

Windows-only project. You need a Windows machine (or VM) with:

- **Rust stable** — `rustup update stable`
- **Node.js ≥ 18** — only needed for `cargo tauri` CLI tooling
- **cargo-tauri** — `cargo install tauri-cli`
- **WebView2 runtime** — ships with Windows 10 1803+; already present on most systems

## Setup

```bat
git clone https://github.com/episuarez/killport
cd killport
cargo build --workspace        # check everything compiles
```

## Dev loop

```bat
scripts\dev.bat                # Tauri hot-reload (Rust + UI)
cargo run -p killport-cli      # headless CLI
cargo test --workspace         # all tests
```

## Code style

```bat
cargo fmt --all                # format
cargo clippy --workspace --all-targets -- -D warnings   # lint, must be clean
```

No warnings allowed on CI. Fix clippy before pushing.

## Commit format

[Conventional Commits](https://www.conventionalcommits.org/). Subject ≤ 50 chars. Body explains *why*, not what.

```
feat: detect Bun runtime on port scan
fix: panic on short MySQL greeting
docs: add setup instructions
chore: bump sysinfo to 0.34
```

No references to AI tools in commit messages.

## Branch model

- `main` — stable, always green
- Feature branches → PR → squash merge to main

## Pull requests

- Describe *what* and *why* in the PR body
- All CI checks must pass (fmt → clippy → test)
- One logical change per PR; split unrelated fixes into separate PRs

## Releases

Releases are tag-driven. When a commit on `main` is ready to ship:

1. Update `version` in the root `Cargo.toml` `[workspace.package]` section
2. Commit: `chore: bump version to X.Y.Z`
3. Push the commit
4. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`

The `release` workflow picks up the tag, builds NSIS + MSI installers, and creates a draft GitHub Release. Review and publish from the Releases page.

The tag must match the version in `Cargo.toml` exactly — the release job validates this before building.

## Scope rules

- Logic stays in `killport-core` — no Tauri or UI imports there
- IPC commands in `src-tauri/src/commands.rs` are thin wrappers; no business logic
- UI is vanilla HTML/CSS/JS — no framework, no bundler
- New dependencies need justification in the PR description
