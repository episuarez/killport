# Process Restart

**Module:** `crates/killport-core/src/restart.rs`

Kills a process and respawns it from its original command line and working directory.

## How it works

```
1. Capture cmd + cwd from PortProcess  ← BEFORE killing (data gone after)
2. kill_tree(pid, Graceful)
3. restart(cmd, cwd)
   └── Command::new(argv[0]).args(argv[1..]).current_dir(cwd).spawn()
```

The critical ordering constraint: `cmd` and `cwd` come from the live `PortProcess`. Once the process is killed those fields are gone from the OS. The scan result must be captured first.

## Windows flag

`CREATE_NO_WINDOW (0x0800_0000)` is set via `CommandExt::creation_flags`. This prevents a new terminal window from flashing on screen when the dev server respawns.

## Limitations

- Requires `cmd` to be non-empty. Processes with no captured command line (some system services) cannot be restarted.
- The respawned process inherits the environment of the KillPort process, not the original one. If the original process had custom env vars set by a shell profile, they may be missing.
- No wait/confirmation that the new process successfully bound the port — the caller should poll via `scan()`.

## Tauri IPC

```
restart_port(pid: u32) -> bool
```

Returns `true` if `spawn()` succeeded. Does not wait for the port to come back up.

## CLI

```
killport restart <port>
```

Finds all processes on `<port>`, kills each gracefully, respawns each from its captured cmd+cwd.
