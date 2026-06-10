# Autostart

**Module:** `crates/killport-core/src/autostart.rs`

Registers KillPort to launch at Windows login via the user registry run key.

## Registry location

```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
Value name: "KillPort"
Value data: "<path to KillPort.exe>"
```

Using `HKCU` (not `HKLM`) means:
- No admin elevation required
- Only affects the current user
- Survives Windows updates without issues

## API

```rust
pub fn set_autostart(enabled: bool) -> bool
pub fn get_autostart() -> bool
```

`set_autostart(true)` writes the registry value.  
`set_autostart(false)` deletes it.  
Both return success/failure.

`get_autostart()` checks if the value exists and points to the current executable.

## Tauri IPC

```
set_autostart(enabled: bool) -> bool
get_autostart() -> bool
```

Called from the settings panel in the desktop UI. The toggle reflects the current registry state on open.

## No CLI equivalent

Autostart is a desktop app concern. The CLI has no autostart commands.
