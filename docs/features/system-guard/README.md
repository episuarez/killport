# System Guard

**Module:** `crates/killport-core/src/guard.rs`

Prevents killing critical system processes.

## Two-layer protection

**Layer 1 — PID floor:**
```rust
const MIN_KILLABLE_PID: u32 = 10;
```
PIDs 0–9 are never touched. PID 4 is the Windows kernel (`System`), PID 0 is the idle process.

**Layer 2 — name list:**
`guard::is_protected(name)` returns `true` for known system process names (e.g. `lsass.exe`, `csrss.exe`, `winlogon.exe`, `svchost.exe` handling core services, etc.).

Both checks run in `kill_one()` before any Win32 call is made.

## is_system flag

`is_system` on `PortProcess` is a broader flag than the guard list. It marks processes whose exe lives under `%WINDIR%` or whose exe path is unknown. `is_system = true` processes:
- Are hidden by default in the CLI (`list` without `--all`)
- Are hidden or visually separated in the desktop UI

`is_system` does not block killing — it's a display filter. The guard is the actual kill gate.

## Why separate concerns

`is_system` answers "should we show this in a dev tool?" (conservative, hides many things).  
`guard::is_protected` answers "is it safe to kill this?" (narrow, only blocks truly dangerous targets).

A process can be `is_system = true` but not protected (e.g. a non-critical Windows background service you actually want to stop). The kill will succeed with a warning surfaced via the `is_system` flag.
