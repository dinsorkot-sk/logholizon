# LOGHOLIZON Desktop (Tauri)

Offline desktop app: the Nuxt frontend runs as a static SPA inside a Tauri
window, and `logholizon-core` boots in-process on an ephemeral loopback port
with its SQLite database under the OS app-data directory. No network, no
external server, no Node runtime in the bundle.

## Architecture

```text
Tauri window (static SPA from packages/app/.output/public)
  -> fetch http://127.0.0.1:<ephemeral>/v1/* + Authorization: Bearer <token>
In-process Axum sidecar (logholizon-core http::router)
  -> SQLite (<app-data>/logholizon/core.db, WAL)
```

- `src-tauri/src/lib.rs` — sidecar lifecycle (`boot`, `wait_for_health`,
  `shutdown`, `probe_runtime`); feature-independent and covered by tests.
- `src-tauri/src/main.rs` — binary: boot + health-gate, then the Tauri
  window (`ui` feature) or headless boot proof (default).
- `packages/core/src/desktop.rs` — shared boot helpers: app-data DB URL,
  desktop `Config` (Tauri origins, background loops off by default),
  `boot_desktop_pool`, `ensure_first_run` (admin + demo seed).
- `packages/app/modules/desktop.ts` — Nuxt module active only with
  `LOGHOLIZON_DESKTOP=1`: replaces `#build/fetch.mjs` so the global
  `$fetch` (every `useFetch`/`$fetch` call site, zero page rewrites)
  routes `/api/*` to the sidecar core.
- `packages/app/app/utils/desktop-routes.ts` — gateway→core route map
  (mirrors `server/api/**` → `/v1/**`).
- `packages/app/app/utils/desktop-fetch.ts` — fetch wrapper + token store
  (`localStorage`, `lh_desktop_token`).
- `packages/app/app/composables/useAuth.ts` — persists the login token to
  the desktop store (no-op on web, where the `lh_session` cookie rules).

## Commands

```bash
# Desktop static frontend (SPA, no Nitro server)
pnpm --dir packages/app run build:desktop
pnpm --dir packages/app run dev:desktop

# Shell gates (no WebKit needed; ui feature off)
cargo test -p logholizon-desktop --no-default-features
cargo clippy -p logholizon-desktop --no-default-features --all-targets -- -D warnings

# Full Tauri dev/build (needs WebKit system libs + cargo-tauri)
cargo install tauri-cli --version '=2.11.5'
cargo tauri dev
cargo tauri build
```

## Linux prerequisites (Debian/Ubuntu/WSL)

The `ui` feature links GTK/WebKit/AppIndicator natively, so `cargo tauri
dev` fails with `pkg-config` / `glib-sys` / `gobject-sys` errors until the
system dev packages are installed (one-time, requires sudo):

```bash
sudo apt update
sudo apt install -y build-essential curl wget file pkg-config libssl-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libxdo-dev
```

Notes:

- `libayatana-appindicator3-dev` is required because the `tauri` dependency
  enables the `tray-icon` feature (`src-tauri/Cargo.toml`).
- Ubuntu 22.04 ships `libwebkit2gtk-4.0-dev` instead of `-4.1-dev`; run
  `apt-cache search libwebkit2gtk` and install whichever variant exists.
- Verify with `pkg-config --exists glib-2.0 gtk+-3.0 webkit2gtk-4.1 && echo OK`.
- WSL also needs WSLg for the GUI window to display; without it, use the
  web-only fallback (`pnpm --dir ../app run dev`) for interactive UI work.

Database: `$XDG_DATA_HOME/logholizon-desktop/logholizon/core.db` on Linux
(`%APPDATA%` on Windows, `~/Library/Application Support` on macOS).
Backups (`VACUUM INTO`) go next to it; restore still stages
`restore-pending.db` and applies on next boot.

## Rules

- Rust owns persistence and domain logic; the shell only boots core.
- No SQL outside `packages/core`; no new `/v1` routes for desktop.
- Web output must stay byte-identical: the desktop module is inert without
  `LOGHOLIZON_DESKTOP=1` (verified: web server bundle has no desktop code).
- `admin/restart` exits the sidecar process; on desktop the shell should
  relaunch rather than rely on an external supervisor.
- Icons: PNG placeholders ship in `src-tauri/icons/`; generate `.icns`
  (macOS) and `.ico` (Windows) with platform tooling before store release.

## Scope boundaries (v0.1.1)

Done: offline boot, sidecar lifecycle, direct `/v1` SPA, token store,
Solution Library from bundled JSON, first-run admin + demo seed.
Deferred to v0.1.2+: auto-updater + signing, system tray, multi-window,
file associations, `.icns`/`.ico` packaging.
