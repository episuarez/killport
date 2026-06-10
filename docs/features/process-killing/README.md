# Process Killing

**Module:** `crates/killport-core/src/kill.rs`

Two kill modes, two entry points.

## KillMode

```rust
pub enum KillMode {
    Graceful,  // WM_CLOSE → wait 1500ms → TerminateProcess
    Force,     // TerminateProcess immediately
}
```

## kill(pid, mode)

Kills a single process. Returns `true` if the process is gone after the call.

**Graceful path:**
1. `EnumWindows` → collect all HWNDs owned by `pid`
2. `PostMessageW(hwnd, WM_CLOSE, 0, 0)` for each window
3. Poll `WaitForSingleObject` every 50ms for up to 1500ms
4. If still alive after timeout → fall through to force

**Force path:**
- `OpenProcess(PROCESS_TERMINATE)` → `TerminateProcess(handle, 1)`

Console-only processes (dev servers with no window) have no HWNDs, so the graceful path finds nothing and falls straight to `TerminateProcess`.

## kill_tree(pid, mode)

Kills `pid` and all its descendants. Returns count of terminated processes.

Why it matters: `npm run dev` spawns Node as a child. Killing only the npm process leaves Node (and the port) alive.

**Algorithm:**
1. `snapshot()` → build `parent → Vec<children>` map
2. BFS from root `pid` → collect full subtree in discovery order
3. Reverse the list (deepest nodes first)
4. `kill_one()` each, root last

Killing leaves first prevents orphan processes from re-attaching to init.

## Safety

- PIDs < 10 are never killed (`MIN_KILLABLE_PID = 10`)
- `guard::is_protected(name)` blocks known system processes
- Both checks happen before any Win32 call

## Tauri IPC

```
kill_port(pid: u32) -> usize
```

Calls `kill_tree(pid, KillMode::Graceful)` and returns the count of terminated processes. The UI uses this count to show feedback.
