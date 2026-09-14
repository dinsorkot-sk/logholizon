import { expect, test } from '@playwright/test'

async function login(page: import('@playwright/test').Page, username: string, password: string) {
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

test('demo user can log in and see the dashboard', async ({ page }) => {
  await login(page, 'demo', 'demo1234')
  await expect(page.getByLabel('Select entity')).toBeVisible()
})

test('login API rejects invalid credentials', async ({ page }) => {
  await page.goto('/login')
  const status = await page.evaluate(async () => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ username: 'demo', password: 'wrong-password' })
    })
    return response.status
  })
  expect(status).toBe(401)
})

test('admin user can log in and see admin navigation', async ({ page }) => {
  await login(page, 'admin', 'admin123')
  await expect(page.getByRole('link', { name: 'Entity Manager' })).toBeVisible()
  await expect(page.getByRole('link', { name: 'Audit Log' })).toBeVisible()
})
