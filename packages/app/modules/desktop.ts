// Desktop Nuxt module: route `/api/*` calls directly to the in-process
// Rust core when building the Tauri desktop SPA.
//
// The web app serves `/api/*` from the Nitro gateway. The desktop static
// build has no Nitro server, so this module replaces the generated
// `#build/fetch.mjs` template: the exported `$fetch` becomes a
// desktop-aware instance that rewrites `/api/*` to the sidecar core `/v1/*`
// endpoints (see `app/utils/desktop-routes.ts`) and attaches the stored
// Bearer token. Non-API URLs fall through to the original ofetch instance.
//
// Enabled only when `LOGHOLIZON_DESKTOP=1` so web dev/build output is
// byte-identical to before. Plain function module (no @nuxt/kit needed).
export default function desktopModule(_options: Record<string, unknown>, nuxt: any) {
  if (process.env.LOGHOLIZON_DESKTOP !== '1' && process.env.NUXT_DESKTOP !== '1') {
    return
  }
  nuxt.hook('app:templates', (app: any) => {
    const template = app.templates.find((t: any) => t.filename === 'fetch.mjs')
    if (!template) return
    template.getContents = () => [
      "import { $fetch as _$fetch } from 'ofetch'",
      "import { baseURL } from '#internal/nuxt/paths'",
      "import { createDesktopFetch } from '../app/utils/desktop-fetch'",
      "import { solutionCatalog, solutionPackage } from '#shared/utils/solution-catalog'",
      'const __baseFetch = _$fetch.create({',
      '  baseURL: baseURL()',
      '})',
      'if (!globalThis.$fetch) {',
      '  globalThis.$fetch = __baseFetch',
      '}',
      '// Desktop: wrap the shared instance so every useFetch/$fetch call',
      '// to /api/* targets the sidecar core directly (no Nitro gateway).',
      'globalThis.$fetch = createDesktopFetch(globalThis.$fetch, {',
      '  catalog: () => solutionCatalog(),',
      '  package: (name) => solutionPackage(name)',
      '})',
      'export const $fetch = globalThis.$fetch',
      ''
    ].join('\n')
  })
}
