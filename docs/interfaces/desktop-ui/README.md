# Desktop UI

**Crate:** `src-tauri` + `ui/`

Tauri 2 desktop app with two surfaces: a system tray popup and a main dashboard window.

## Surfaces

### Tray popup (`ui/tray.html`)

Shown when clicking the system tray icon. Compact list of active dev ports.

Per-port entry shows:
- Port number + URL
- Framework / kind label
- Project name (if detected)
- Quick actions: Kill, Open in browser, Copy URL

Managed by `ui/scripts/tray.js`. Polls via `scan_fast()` for low-latency updates.

### Main dashboard (`ui/index.html`)

Full window with more detail and configuration options.

Shows:
- Complete port list (more columns than tray)
- Process details: PID, cmd, cwd, CPU/mem
- Container and service info
- Actions: Kill, Restart, Open folder, Open in editor
- Settings panel: autostart toggle, configuration

Managed by `ui/scripts/app.js`. Uses `scan()` (full enrichment) on load and manual refresh.

## Tauri IPC commands

All frontend↔backend communication goes through Tauri's `invoke()` API:

| Command | Returns | Description |
|---------|---------|-------------|
| `list_ports` | `Vec<PortProcess>` | Full scan |
| `kill_port(pid)` | `usize` | Kill tree, return count |
| `restart_port(pid)` | `bool` | Kill + respawn |
| `open_url(port)` | — | Launch browser |
| `copy_url(port)` | — | Copy to clipboard |
| `open_folder(path)` | — | Open in Explorer |
| `open_editor(path)` | — | Open in VS Code / editor |
| `get_config` | `Config` | Read current config |
| `set_config(cfg)` | — | Write config |
| `get_autostart` | `bool` | Registry check |
| `set_autostart(bool)` | `bool` | Registry write |
| `open_main(app)` | — | Show main window |
| `hide_tray(app)` | — | Hide tray popup |
| `quit_app(app)` | — | Exit |

## Frontend stack

- HTML5 + CSS3 + Vanilla JavaScript — no framework
- `ui/styles/tokens.css` — design system variables (colors, spacing, type scale)
- `ui/styles/app.css` — main dashboard styles
- `ui/styles/tray.css` — tray popup styles

## Window management

Both windows are managed by Tauri. The tray popup is a borderless always-on-top window that shows/hides on tray icon click. The main window is a standard resizable frame.

## Build

```
cargo tauri build
```

Produces a Windows installer (NSIS) + portable exe in `target/release/bundle/`.

## Capabilities

Security capabilities are defined in `src-tauri/capabilities/default.json`. Current setup allows the frontend to call the listed IPC commands and access localhost URLs.
