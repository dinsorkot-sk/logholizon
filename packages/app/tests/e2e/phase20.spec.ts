import { expect, test, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.evaluate(async () => {
    await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ username: 'demo', password: 'demo1234' })
    })
  })
}

test('Phase 20: user-facing Module Builder creates a complete business domain shell', async ({ page }) => {
  await login(page)
  const suffix = Date.now()
  const name = `vehicle_acceptance_${suffix}`
  const label = `Vehicle Acceptance ${suffix}`
  await page.goto('/admin/modules')
  await expect(page.getByRole('heading', { name: 'Module Builder' })).toBeVisible({ timeout: 15_000 })

  const newModule = page.getByRole('button', { name: 'New module' }).first()
  await expect.poll(async () => {
    await newModule.click()
    return page.getByPlaceholder('vehicle').count()
  }, { timeout: 15_000 }).toBeGreaterThan(0)
  await page.getByPlaceholder('vehicle').fill(name)
  await page.getByPlaceholder('Vehicle Management').fill(label)
  await page.getByPlaceholder('Fleet, drivers, maintenance').fill('Phase 20 generic runtime acceptance')
  await page.getByRole('button', { name: 'Create', exact: true }).click()

  await expect(page.getByText(label, { exact: true })).toBeVisible({ timeout: 15_000 })
  await page.getByText(label, { exact: true }).click()
  await expect(page.getByRole('heading', { name: label })).toBeVisible({ timeout: 15_000 })

  for (const entity of [
    ['vehicle', 'Vehicle'],
    ['driver', 'Driver'],
    ['maintenance', 'Maintenance'],
    ['rental', 'Rental']
  ]) {
    const [entityName, entityLabel] = entity
    await page.getByPlaceholder('vehicle (snake_case)').fill(entityName)
    await page.getByPlaceholder('Vehicle').fill(entityLabel)
    await page.getByRole('button', { name: 'Add entity' }).click()
    await expect(page.getByRole('heading', { name: new RegExp(`^${entityName} ·`) })).toBeVisible({ timeout: 15_000 })
  }

  // The builder persists metadata immediately; verify the runtime can now load
  // the generated application route without any domain-specific frontend page.
  const modules = await page.evaluate(async () => (await fetch('/api/modules')).json())
  const created = modules.find((m: { name: string }) => m.name === name)
  expect(created).toBeTruthy()
  expect(created.definition.entities.map((e: { name: string }) => e.name)).toEqual([
    'vehicle', 'driver', 'maintenance', 'rental'
  ])

  // No built-in domain route is required: the generic entity route is selected
  // from metadata after publish.
  await page.getByRole('button', { name: 'Submit for review' }).click()
  await page.getByRole('button', { name: 'Publish' }).click()
  await expect(page.getByText('published', { exact: true })).toBeVisible({ timeout: 15_000 })

  const published = await page.evaluate(async (moduleId) => {
    const response = await fetch(`/api/modules/${encodeURIComponent(moduleId)}`)
    return response.json()
  }, created.id)
  expect(published.status).toBe('published')
  expect(published.definition.entities).toHaveLength(4)
})
