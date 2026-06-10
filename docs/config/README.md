# Configuration

**Module:** `crates/killport-core/src/config.rs`

User preferences persisted in a TOML file.

## Location

Standard Windows app data directory:
```
%APPDATA%\KillPort\config.toml
```

## Config struct

```rust
pub struct Config {
    // fields TBD as features are added
}
```

## Access

Config is loaded at app start and held in `Mutex<Config>` in Tauri's app state:

```rust
struct AppState {
    config: Mutex<Config>,
}
```

All IPC handlers that read or write config lock this mutex.

## Tauri IPC

```
get_config() -> Config
set_config(cfg: Config)
```

The settings panel in the main window reads `get_config` on open and writes `set_config` on save. Changes persist immediately to disk.

## CLI

The CLI does not expose config commands. It uses `scan()`/`kill()`/`restart()` directly with no user-configurable options beyond command-line flags.
