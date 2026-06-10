# Windows Services

**Module:** `crates/killport-core/src/service.rs`

Identifies whether a listening process is a registered Windows service.

## How it works

Queries the Windows Service Control Manager (SCM) to build a `HashMap<u32, String>` mapping `pid → service_name`. During `scan()`, each `PortProcess` gets its `service` field set if its PID appears in that map.

## Why it matters

Service-owned processes behave differently from ad-hoc dev processes:
- They may restart automatically (via SCM recovery actions)
- Killing them may be ineffective or produce warnings
- The correct stop mechanism is `sc stop <service>` or the Services MMC snap-in, not `TerminateProcess`

The `service` field lets the UI display a warning or different action when the process is a Windows service.

## Display

When `service` is `Some(name)`:
- CLI `list` shows `service: <name>` in the ORIGIN column
- Desktop UI shows the service name as context

## When it runs

Only in `scan()`. `scan_fast()` skips SCM queries.

## No special kill path

KillPort does not call `ControlService(SERVICE_CONTROL_STOP)` — it uses the same `kill_tree` path as any other process. This is intentional: the tool targets dev workflows where even service-wrapped processes (e.g. PostgreSQL installed as a service) need a quick kill. The service name is surfaced as information, not a gate.
