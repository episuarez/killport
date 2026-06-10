# Framework Detection

**Module:** `crates/killport-core/src/framework.rs`

Best-effort detection of the web framework running on a port. Used for display only — never affects kill logic.

## Detection strategy

Two-pass approach:

**Pass 1 — command line (authoritative):** scan the full joined command line for framework-specific tokens.

**Pass 2 — port hint (fallback):** if no command line signal found, use the port number as a weak hint.

Command line always wins over port hint.

## Supported frameworks (command line)

| Label | Token in cmd |
|-------|-------------|
| Vite | `vite` |
| Next.js | `next` |
| Nuxt | `nuxt` |
| Astro | `astro` |
| Remix | `remix` |
| Gatsby | `gatsby` |
| SvelteKit | `sveltekit` or `svelte-kit` |
| Create React App | `react-scripts` |
| Angular | `angular` or `@angular` |
| Webpack | `webpack` |
| Vue CLI | `vue-cli-service` |
| FastAPI/Uvicorn | `uvicorn` or `fastapi` |
| Flask | `flask` |
| Django | `django` or `manage.py` |
| Gunicorn | `gunicorn` |
| Laravel | `artisan` |
| Rails | `rails` or `puma` |
| Express | `express` |

## Port hints (fallback only)

| Port | Hint |
|------|------|
| 3000 | Node / Next.js / CRA |
| 3001 | Node (alt) |
| 4200 | Angular |
| 5000 | Flask / .NET |
| 5173 | Vite |
| 5432 | PostgreSQL |
| 6379 | Redis |
| 8000 | Django / Python |
| 8080 | HTTP / Webpack |
| 8888 | Jupyter |
| 9000 | PHP / SonarQube |
| 27017 | MongoDB |

## Return value

`Option<String>` — `None` when no signal found. The UI omits the framework column when `None`.

## Tests

`framework.rs` has unit tests for: cmd beats port (Vite on :3000), port-hint fallback (Node cmd on :5173), and no-match → None.
