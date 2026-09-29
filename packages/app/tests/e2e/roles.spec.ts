import { expect, test, type Page } from '@playwright/test'

async function login(page: Page, username: string, password: string) {
  // Log in through the API: see auth.spec.ts for why the UI form is avoided.
  await page.goto('/login')
  await page.evaluate(async ([user, pass]) => {
    await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ username: user, password: pass })
    })
  }, [username, password])
  // A cold Nuxt dev server may reload the page while Vite optimizes deps,
  // aborting the first navigation (net::ERR_ABORTED). Retry until stable.
  await expect(async () => {
    await page.goto('/dashboard')
    await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 15_000 })
  }).toPass({ timeout: 90_000 })
}

async function typeInto(page: Page, locator: ReturnType<Page['getByPlaceholder']>, value: string) {
  // UInput v-model needs real key events; fill() is not picked up reliably.
  await locator.click()
  await page.keyboard.type(value, { delay: 10 })
}

test('admin can create and delete a custom role from the Roles page', async ({ page }) => {
  await login(page, 'admin', 'admin123')
  await page.goto('/admin/roles')
  await expect(page.getByRole('button', { name: 'New role' })).toBeVisible({ timeout: 15_000 })

  const suffix = Date.now().toString(36)
  const name = `e2e_role_${suffix}`
  const label = `E2E Role ${suffix}`

  // Headless Chromium may need a second activation to open overlays; poll.
  const newRole = page.getByRole('button', { name: 'New role' })
  await expect.poll(async () => {
    await newRole.click()
    return page.getByRole('dialog').count()
  }, { timeout: 15_000 }).toBeGreaterThan(0)

  const dialog = page.getByRole('dialog')
  await typeInto(page, dialog.getByPlaceholder('sales_manager'), name)
  await typeInto(page, dialog.getByPlaceholder('Sales Manager'), label)
  await dialog.getByRole('button', { name: 'Create role' }).click()

  await expect(page.getByText('Role created', { exact: true })).toBeVisible({ timeout: 15_000 })
  const row = page.locator('div.flex.items-center.justify-between', { hasText: label })
  await expect(row).toBeVisible()
  await expect(row.getByText(name, { exact: true })).toBeVisible()

  // Custom roles are deletable; system roles expose no Delete button.
  await row.getByRole('button', { name: 'Delete' }).click()
  await expect(page.getByText('Role deleted', { exact: true })).toBeVisible({ timeout: 15_000 })
  await expect(page.getByText(label)).toHaveCount(0)

  // The role is gone in core too, not just hidden in the UI.
  const roles = await page.evaluate(async () => (await fetch('/api/admin/roles')).json())
  expect((roles as Array<{ name: string }>).some(role => role.name === name)).toBe(false)
})

test('role API validates input and is admin-only', async ({ page }) => {
  await login(page, 'admin', 'admin123')
  const invalid = await page.request.post('/api/admin/roles', { data: { name: '', label: '' } })
  expect(invalid.status()).toBe(400)

  await page.context().clearCookies()
  await login(page, 'demo', 'demo1234')
  const denied = await page.request.post('/api/admin/roles', { data: { name: 'e2e_denied', label: 'Denied' } })
  expect(denied.status()).toBe(403)
})
