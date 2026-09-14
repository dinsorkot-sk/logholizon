# LOGHOLIZON App

Nuxt 4 + Nuxt UI frontend and thin Nitro gateway for the LOGHOLIZON ERP platform.

See the [root README](../../README.md) for full setup, commands, and architecture.

## Development

```bash
pnpm install
pnpm dev
```

The app expects the Rust core on `http://127.0.0.1:8787` (override with `CORE_URL`).

## Structure

- `app/pages/` — UI pages (dashboard, entity lists, admin)
- `app/composables/` — shared state (auth)
- `app/utils/` — shared helpers (validation, audit time)
- `server/api/` — thin Nitro gateway routes (parse → validate → call core)
- `server/core/client.ts` — single HTTP client to the Rust core

## Gates

```bash
pnpm run build
pnpm run test
pnpm run check
```

## Production

Build the application for production:

```bash
# npm
npm run build

# pnpm
pnpm build

# yarn
yarn build

# bun
bun run build
```

Locally preview production build:

```bash
# npm
npm run preview

# pnpm
pnpm preview

# yarn
yarn preview

# bun
bun run preview
```

Check out the [deployment documentation](https://nuxt.com/docs/getting-started/deployment) for more information.
