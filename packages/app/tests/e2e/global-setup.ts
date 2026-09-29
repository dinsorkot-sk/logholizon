import { chromium, type FullConfig } from '@playwright/test'

// A cold Nuxt dev server optimizes Vite deps on first page load and then
// full-reloads the page, which aborts the first navigations (ERR_ABORTED) of
// whichever test happens to run first. Warm the main routes once up front so
// every spec starts against a stable server.
export default async function globalSetup(config: FullConfig) {
  const baseURL = config.projects[0]?.use.baseURL
  if (!baseURL) return
  const browser = await chromium.launch()
  const page = await browser.newPage({ baseURL })
  const routes = ['/login', '/dashboard', '/app/work_order', '/admin/roles', '/admin/users']
  try {
    await page.goto('/login')
    await page.evaluate(async () => {
      await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ username: 'admin', password: 'admin123' })
      })
    })
    for (const route of routes) {
      for (let attempt = 0; attempt < 5; attempt++) {
        try {
          await page.goto(route, { waitUntil: 'networkidle', timeout: 60_000 })
          break
        } catch {
          // Aborted by a Vite dep-optimization reload; retry.
        }
      }
    }
  } finally {
    await browser.close()
  }
}
