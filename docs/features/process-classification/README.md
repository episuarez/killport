# Process Classification

**Module:** `crates/killport-core/src/classify.rs`

Maps a process to a `kind` string used for display, filtering, and icon selection.

## Classification logic

Priority order (first match wins):

| kind | Signal |
|------|--------|
| `postgresql` | exe name contains `postgres` |
| `mysql` | exe name contains `mysqld`, `mysql.exe`, or `mariadb` |
| `redis` | exe name contains `redis` |
| `mongodb` | exe name contains `mongod` |
| `sqlserver` | exe name contains `sqlservr` |
| `docker` | exe name contains `docker`, `com.docker`, or `vpnkit` |
| `wsl` | exe name contains `wslrelay`, `wslhost`, or is `wsl.exe` |
| `vite` | is_node AND cmd contains `vite` |
| `next.js` | is_node AND cmd contains `next` |
| `node` | exe is node / cmd has npm / pnpm / yarn / bun |
| `python` | exe contains `python` OR cmd has flask/django/uvicorn/gunicorn |
| `php` | exe contains `php` OR cmd has `artisan` |
| `unknown` | none of the above |

`is_node` is computed before checking framework subkinds. This prevents false positives like `NextDNS.exe` matching as `next.js` — the cmd check for `next` only runs when the process is already identified as Node.

## Design notes

- Databases and services are matched by **exe name** (most reliable signal)
- Framework detection within Node uses **command line** (more specific than exe)
- `vite` and `next.js` are returned as distinct `kind` values, not just `node`, to allow icon differentiation and smarter default actions
- `unknown` is intentional — it's used by the UI and CLI to filter out unrecognized processes by default

## Tests

`classify.rs` has unit tests covering: node from cmd, vite over node, python, postgres, redis, unknown fallback, and the NextDNS false-positive guard.
