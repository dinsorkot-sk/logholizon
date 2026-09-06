import { expect, test, type Page } from '@playwright/test'

async function login(page: Page, username: string, password: string) {
  // Log in through the API: UAuthForm's vee-validate state does not pick up
  // Playwright-filled values (submit sees empty fields), so driving the UI
  // form is flaky. The cookie set here exercises the real auth flow.
  await page.goto('/login')
  await page.evaluate(async ([user, pass]) => {
    await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ username: user, password: pass })
    })
  }, [username, password])
  await page.goto('/dashboard')
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 15_000 })
}

test('admin can view users and audit log', async ({ page }) => {
  await login(page, 'admin', 'admin123')

  await page.goto('/admin/users')
  await expect(page.getByText('demo')).toBeVisible({ timeout: 15_000 })

  await page.goto('/admin/audit')
  // Audit table shows document IDs (e.g. demo-wo-1), not payload titles.
  await expect(page.getByText('demo-wo-1').first()).toBeVisible({ timeout: 15_000 })
})

test('demo user cannot reach admin API', async ({ page }) => {
  await login(page, 'demo', 'demo1234')
  const response = await page.request.get('/api/meta/entities')
  expect(response.status()).toBe(403)
})
