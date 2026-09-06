import { defineConfig, devices } from '@playwright/test'

const appPort = 3100

export default defineConfig({
  testDir: './tests/e2e',
  testIgnore: '**/run.cjs',
  fullyParallel: false,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    // Nuxt dev binds IPv6 loopback (`[::1]`) on Windows; `localhost` resolves
    // there while `127.0.0.1` is refused. Core binds IPv4, so keep its URL.
    baseURL: process.env.PLAYWRIGHT_BASE_URL || `http://localhost:${appPort}`,
    trace: 'on-first-retry'
  },
  workers: 1,
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }]
})
