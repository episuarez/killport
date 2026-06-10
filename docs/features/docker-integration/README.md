# Docker Integration

**Module:** `crates/killport-core/src/docker.rs`

Maps listening ports to Docker container names when Docker Desktop is running.

## How it works

Connects to the Docker Engine API via the named pipe `\\.\pipe\docker_engine` and calls the containers list endpoint. For each running container, maps its published ports (host port → container name).

The result is a `HashMap<u16, String>` (host port → container name) used during `scan()` to set `PortProcess::container`.

## When it runs

Only in `scan()` (full scan). `scan_fast()` skips Docker to avoid the named pipe overhead during high-frequency polling.

## Graceful degradation

If Docker Desktop is not running or the named pipe is unavailable, `container_map()` returns an empty `HashMap`. Processes that would have had a container name show `None` in that field — the rest of the scan is unaffected.

## Display

When `container` is `Some`, the UI shows the container name instead of (or alongside) the process name. This makes it obvious you're looking at a Dockerized service, not a native process.

## Killing Docker processes

Killing a Docker-mapped process via `kill_tree` terminates the proxy/relay process on the host. This effectively stops the container's port binding but does not stop the container itself. To stop the container cleanly, use `docker stop <name>` directly.
