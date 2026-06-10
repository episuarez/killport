# Port Scanning

**Module:** `crates/killport-core/src/ports.rs`, `scan.rs`

Enumerates all TCP ports currently in LISTEN state and enriches each with process metadata.

## How it works

Port enumeration uses `GetExtendedTcpTable` (Win32 IP Helper API) directly — not `netstat` parsing. This gives kernel-level accuracy with no subprocess overhead.

Steps:
1. Call `GetExtendedTcpTable` for IPv4 (`TCP_TABLE_OWNER_PID_LISTENER`)
2. Call `GetExtendedTcpTable` for IPv6 (`TCP_TABLE_OWNER_MODULE_LISTENER`)
3. Convert network byte order → host `u16` for the port
4. Collect `(port, pid)` pairs, deduplicate

## Enrichment pipeline

Raw `(port, pid)` pairs go through `scan_inner()` which joins with:

| Source | Data added |
|--------|-----------|
| `sysinfo` snapshot | name, cmd, cwd, exe, cpu, mem, parent pid |
| `classify()` | kind (node/python/postgresql/…) |
| `framework::detect()` | framework label (Vite, Next.js, Django, …) |
| `project::detect()` | project name + path from git/workspace |
| `appinfo::app_name()` | human name from exe version resource |
| `service_map()` | Windows service name if process is a service |
| `container_map()` | Docker container name if port maps to one |
| `is_system()` | flag for OS/system processes |

## scan() vs scan_fast()

| | `scan()` | `scan_fast()` |
|---|---------|--------------|
| Docker lookup | yes | no |
| Service map | yes | no |
| App name (exe version) | yes | no |
| Classification | yes | yes |
| Framework detection | yes | yes |
| Project detection | yes | yes |
| is_system / parent_name | yes | yes |

`scan_fast()` is used for high-frequency polling in the UI (change detection). `scan()` is used on initial load and manual refresh.

## Output

```rust
pub struct PortProcess {
    pub port: u16,
    pub pid: u32,
    pub name: String,
    pub url: String,   // always "http://localhost:{port}"
    // + all enriched fields
}
```

Results are sorted ascending by port number.

## System process detection

A process is flagged `is_system = true` when:
- Its exe path starts with `%WINDIR%` (e.g. `C:\Windows\`)
- Or it appears in the guard protection list
- Or its exe path is unknown (no access)

System processes are hidden by default in both CLI (`list` without `--all`) and the desktop UI.
