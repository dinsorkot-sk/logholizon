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
  const createDialog = page.getByRole('dialog')
  await expect.poll(async () => {
    await newModule.click()
    return createDialog.getByLabel('Name (snake_case)', { exact: true }).count()
  }, { timeout: 15_000 }).toBeGreaterThan(0)
  await createDialog.getByLabel('Name (snake_case)', { exact: true }).fill(name)
  await createDialog.getByLabel('Label', { exact: true }).fill(label)
  await createDialog.getByLabel('Description', { exact: true }).fill('Phase 20 generic runtime acceptance')
  await createDialog.getByRole('button', { name: 'Create', exact: true }).click()

  await expect(page.getByText(label, { exact: true })).toBeVisible({ timeout: 15_000 })
  const createdBeforeNavigation = await page.evaluate(async (moduleName) => {
    const modules = await fetch('/api/modules').then(response => response.json())
    return modules.find((module: { name: string }) => module.name === moduleName)
  }, name)
  expect(createdBeforeNavigation).toBeTruthy()
  await page.goto(`/admin/modules/${encodeURIComponent(createdBeforeNavigation.id)}`)
  await expect(page.getByText(label, { exact: true }).first()).toBeVisible({ timeout: 15_000 })

  for (const entity of [
    ['vehicle', 'Vehicle'],
    ['driver', 'Driver'],
    ['maintenance', 'Maintenance'],
    ['rental', 'Rental']
  ]) {
    const [entityName, entityLabel] = entity
    const entityBuilder = page.getByTestId('entity-builder')
    const entityNameInput = page.getByLabel('Entity name (snake_case)', { exact: true })
    const entityLabelInput = page.getByLabel('Entity label', { exact: true })
    const addEntityButton = page.getByRole('button', { name: 'Add entity' })
    await expect(entityBuilder).toHaveAttribute('data-saving', 'false', { timeout: 15_000 })
    await entityNameInput.fill(entityName)
    await entityLabelInput.fill(entityLabel)
    await entityLabelInput.press('Tab')
    await expect(entityNameInput).toHaveValue(entityName)
    await expect(addEntityButton).toBeEnabled()
    await addEntityButton.click()
    await expect(entityBuilder).toHaveAttribute('data-saving', 'false', { timeout: 15_000 })
    await expect.poll(async () => {
      const response = await page.request.get(`/api/modules/${encodeURIComponent(createdBeforeNavigation.id)}?fresh=${Date.now()}`, {
        headers: { 'cache-control': 'no-cache' }
      })
      const current = await response.json()
      return current.definition.entities.some((candidate: { name: string }) => candidate.name === entityName)
    }, { timeout: 15_000 }).toBe(true)
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
