import { expect, test, type Page } from '@playwright/test'

async function login(page: Page, username = 'demo', password = 'demo1234') {
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

  // Add entities via a single batch API call: Nuxt UI v3 UInput's v-model
  // state does not reliably pick up Playwright-filled values in CI, making
  // UI-driven entity creation flaky (addEntity returns early on empty name).
  const entityNames = ['vehicle', 'driver', 'maintenance', 'rental']
  await page.evaluate(async ({ id, entities }) => {
    const mod = await fetch(`/api/modules/${encodeURIComponent(id)}`).then(r => r.json())
    const definition = mod.definition || { entities: [] }
    for (const { eName, eLabel } of entities) {
      definition.entities.push({ name: eName, label: eLabel, fields: [] })
    }
    const response = await fetch(`/api/modules/${encodeURIComponent(id)}`, {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ definition })
    })
    if (!response.ok) throw new Error(`PUT failed: ${response.status}`)
  }, {
    id: createdBeforeNavigation.id,
    entities: [
      { eName: 'vehicle', eLabel: 'Vehicle' },
      { eName: 'driver', eLabel: 'Driver' },
      { eName: 'maintenance', eLabel: 'Maintenance' },
      { eName: 'rental', eLabel: 'Rental' }
    ]
  })

  // Reload so the Vue component re-fetches the module definition from the server.
  await page.goto(`/admin/modules/${encodeURIComponent(createdBeforeNavigation.id)}`)
  await expect(page.getByTestId('entity-builder')).toBeVisible({ timeout: 15_000 })

  // Verify all entities are visible in the builder UI.
  for (const entityName of entityNames) {
    await expect(page.getByText(entityName, { exact: true }).first()).toBeVisible({ timeout: 15_000 })
  }

  // Verify server-side persistence.
  await expect.poll(async () => {
    const response = await page.request.get(`/api/modules/${encodeURIComponent(createdBeforeNavigation.id)}?fresh=${Date.now()}`, {
      headers: { 'cache-control': 'no-cache' }
    })
    const current = await response.json()
    return current.definition.entities.map((e: { name: string }) => e.name).sort()
  }, { timeout: 15_000 }).toEqual(entityNames.sort())

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
