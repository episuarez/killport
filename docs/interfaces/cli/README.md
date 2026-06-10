# CLI Interface

**Crate:** `crates/killport-cli`

Headless binary for scripting and terminal workflows. No GUI, no tray, no notifications.

## Commands

### list

```
killport list [--all | -a]
```

Lists all listening ports. By default hides system processes (`is_system = true`) and `unknown` kind. Use `--all` to show everything.

Output columns:
```
PORT   PID     KIND      APP                    PROJECT                SYS    ORIGIN
3000   18432   node      my-app                 my-project             no     by Code.exe
5432   9120    postgres  PostgreSQL Server       -                      no     service: postgresql-x64-14
```

`ORIGIN` shows one of:
- `service: <name>` — Windows service
- `by <parent>` — launched by a specific process
- `ad-hoc` — no identifiable parent

### kill

```
killport kill <port> [--force]
killport kill --pid <pid> [--force]
```

Kills the process tree on `<port>` or by PID directly. Graceful by default (WM_CLOSE → wait → force). `--force` skips directly to `TerminateProcess`.

Prints:
```
killed 2 process(es) in tree of pid 18432
```

Exits 1 if nothing was found or nothing was terminated (e.g. insufficient privileges).

### restart

```
killport restart <port>
```

Kills the process tree on `<port>` gracefully, then respawns from the captured command line and working directory.

Prints:
```
restarted port 3000 (pid 18432 -> respawned)
```

Exits 1 if nothing is listening on that port. Prints to stderr if spawn fails (no captured command line).

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Nothing found / operation failed |
| 2 | Bad arguments |

## Build

```
cargo build --release -p killport-cli
```

Produces `target/release/killport.exe`.
