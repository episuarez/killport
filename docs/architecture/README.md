# Architecture

## Crate structure

```
KillPort (Cargo workspace)
├── killport-core     # Pure logic, no UI dependency
├── killport-cli      # Thin CLI binary (depends on core)
└── src-tauri         # Tauri app backend (depends on core)
```

`killport-core` is the single source of truth for all port/process logic. Both `killport-cli` and `src-tauri` are thin shells that expose the same underlying operations through different interfaces.

## Layers

```
┌─────────────────────────────────────────────────┐
│  Frontend (HTML/CSS/JS)                         │
├─────────────────────────────────────────────────┤
│  src-tauri  (Tauri IPC commands, notifications) │
│  killport-cli  (arg parsing, table output)      │
├─────────────────────────────────────────────────┤
│  killport-core                                  │
│  ┌──────┬──────────┬────────┬────────┬────────┐ │
│  │ports │ process  │service │ docker │ guard  │ │
│  │scan  │ snapshot │  map   │  map   │ rules  │ │
│  └──────┴──────────┴────────┴────────┴────────┘ │
│  ┌──────────────────────────────────────────┐   │
│  │  classify · framework · project · appinfo │   │
│  └──────────────────────────────────────────┘   │
├─────────────────────────────────────────────────┤
│  Windows Win32 / WinRT APIs                     │
│  (IP Helper, Threading, Services, Registry,     │
│   WindowsAndMessaging, WinRT Notifications)     │
└─────────────────────────────────────────────────┘
```

## Data flow: scan

```
listening_ports()          → Vec<{port, pid}>  (Win32 GetExtendedTcpTable)
      │
      ▼
snapshot()                 → HashMap<pid, ProcInfo>  (sysinfo)
      │
      ├── service_map()    → HashMap<pid, service_name>  (SCM)
      ├── container_map()  → HashMap<port, container_name>  (Docker named pipe)
      │
      ▼
per port+pid:
  classify(name, cmd)      → kind string
  framework::detect(cmd)   → Option<framework>
  project::detect(cwd)     → Option<{name, path}>
  appinfo::app_name(exe)   → Option<human name>
      │
      ▼
Vec<PortProcess>  (sorted by port)
```

`scan()` runs all enrichment steps. `scan_fast()` skips Docker, service map, and app name lookup — used for high-frequency UI polling where those three are the expensive parts.

## Main data structure

```rust
pub struct PortProcess {
    pub port: u16,
    pub pid: u32,
    pub name: String,           // exe name
    pub app: Option<String>,    // human name from exe version resource
    pub kind: String,           // node / python / postgresql / docker / …
    pub framework: Option<String>,
    pub project: Option<String>,
    pub project_path: Option<String>,
    pub service: Option<String>,
    pub container: Option<String>,
    pub parent_name: Option<String>,
    pub is_system: bool,
    pub url: String,            // http://localhost:{port}
    pub cmd: Vec<String>,
    pub cwd: Option<String>,
    pub exe: Option<String>,
    pub cpu: f32,
    pub mem: u64,
}
```

## Kill flow

```
kill_tree(pid, mode)
  │
  ├── snapshot() → build parent→children map
  ├── BFS from pid → collect subtree
  └── iterate in reverse (leaves first)
        └── kill_one(pid, mode)
              ├── [Graceful] windows_of(pid) → post WM_CLOSE to each HWND
              │     └── poll is_alive() every 50ms for up to 1500ms
              └── [Force / timeout] TerminateProcess(pid)
```

## Restart flow

```
capture cmd + cwd from PortProcess  (BEFORE killing — process gone after)
  │
kill_tree(pid, Graceful)
  │
restart(cmd, cwd)
  └── Command::new(program).args(args).current_dir(cwd).spawn()
      with CREATE_NO_WINDOW flag on Windows
```

## Tauri state model

```rust
struct AppState {
    config: Mutex<Config>,
}
```

Single `Mutex<Config>` shared across all IPC command handlers. All commands are async and go through `tauri::State<AppState>`.

## Windows API surface

| Module | Win32 API | Purpose |
|--------|-----------|---------|
| `ports` | `GetExtendedTcpTable` (IP Helper) | Enumerate all TCP listeners |
| `kill` | `EnumWindows`, `PostMessageW`, `WM_CLOSE` | Graceful shutdown |
| `kill` | `OpenProcess`, `TerminateProcess` | Force kill |
| `kill` | `WaitForSingleObject` | Poll for process exit |
| `service` | Service Control Manager APIs | Map pid → service name |
| `autostart` | `HKCU\...\Run` via `winreg` | Boot persistence |
| `notify` | WinRT `ToastNotificationManager` | Toast notifications |
| `appinfo` | `GetFileVersionInfoW`, `VerQueryValueW` | Human app name from exe |
