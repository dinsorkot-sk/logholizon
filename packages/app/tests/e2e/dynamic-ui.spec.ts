import { expect, test, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.evaluate(async () => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ username: 'admin', password: 'admin123' })
    })
    if (!response.ok) throw new Error(`login failed: ${response.status}`)
  })
  await page.goto('/dashboard')
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible({ timeout: 15_000 })
}

test('generic entity UI renders and edits records from metadata', async ({ page }) => {
  await login(page)

  const entity = await page.evaluate(async () => {
    const response = await fetch('/api/entities/work_order')
    if (!response.ok) throw new Error(`entity metadata failed: ${response.status}`)
    return response.json()
  })
  expect(entity.id).toBe('work_order')
  expect(entity.fields.length).toBeGreaterThan(0)

  await page.goto('/app/work_order')
  await expect(page.getByText(entity.label, { exact: true }).first()).toBeVisible({ timeout: 15_000 })

  // The form layout is itself metadata. The demo seed provides a section and
  // explicit field order; the generic record form must render it without an
  // entity-specific layout definition.
  const formFields = entity.fields.filter((field: { hidden?: boolean; can_view?: boolean; is_status?: boolean }) => !field.hidden && (field.can_view ?? true) && !field.is_status)
  expect(formFields.length).toBeGreaterThanOrEqual(2)
  const firstField = formFields.find((field: { id: string }) => field.id === 'work_order_title')
  const secondField = formFields.find((field: { id: string }) => field.id === 'work_order_priority')
  expect(firstField).toBeTruthy()
  expect(secondField).toBeTruthy()

  // Ensure the form-layout metadata has loaded before opening the record
  // dialog.  The section headers are rendered from this data; if the fetch
  // hasn't resolved yet, layoutSections is null and the flat fallback is used.
  await expect.poll(async () => {
    const response = await page.evaluate(async () => (await fetch('/api/entities/work_order/form-layout')).json())
    return response?.config?.sections?.some((s: { label?: string }) => s.label === 'Primary details') ?? false
  }, { timeout: 15_000 }).toBe(true)

  // Reload so the page component's useFetch picks up the available layout
  // data during SSR rather than resolving asynchronously after mount.
  await page.goto('/app/work_order')
  await expect(page.getByText(entity.label, { exact: true }).first()).toBeVisible({ timeout: 15_000 })

  await page.getByRole('button', { name: 'New record' }).click()
  // USlideover may teleport content outside the role="dialog" element, so
  // look for the section header on the page rather than scoping to the dialog.
  await expect(page.getByText('Primary details', { exact: true })).toBeVisible({ timeout: 15_000 })
  const layoutDialog = page.getByRole('dialog')
  const firstLabel = firstField.name
  const secondLabel = secondField.name
  const firstText = page.getByText(firstLabel, { exact: true }).first()
  const secondText = page.getByText(secondLabel, { exact: true }).first()
  await expect(firstText).toBeVisible()
  await expect(secondText).toBeVisible()
  const order = await layoutDialog.locator('label').evaluateAll((labels) => labels.map(label => label.textContent?.trim()).filter(Boolean))
  expect(order.indexOf(secondLabel)).toBeLessThan(order.indexOf(firstLabel))
  await page.getByRole('button', { name: 'Cancel' }).click()

  // Saved views can also persist the table projection. The generic runtime
  // must restore column visibility from view metadata.
  const viewResponse = await page.evaluate(async ({ columns }) => {
    const response = await fetch('/api/entities/work_order/views', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ name: `Metadata columns acceptance ${Date.now()}`, config: { columns, sort_by: 'title', sort_dir: 'asc' } })
    })
    return { status: response.status, body: await response.text() }
  }, { columns: ['title'] })
  expect(viewResponse.status, viewResponse.body).toBe(200)
  const savedView = JSON.parse(viewResponse.body) as { id: string }
  await page.goto(`/app/work_order?view=${encodeURIComponent(savedView.id)}`)
  await expect(page.getByText(entity.label, { exact: true }).first()).toBeVisible({ timeout: 15_000 })
  await expect(page.getByRole('columnheader', { name: 'title' })).toBeVisible()
  await expect(page.getByRole('columnheader', { name: 'priority' })).toHaveCount(0)

  // The table is generated from entity metadata, not a work-order-specific
  // column definition in the page.
  await page.goto('/app/work_order')
  for (const field of entity.fields.filter((field: { hidden?: boolean; can_view?: boolean }) => !field.hidden && (field.can_view ?? true))) {
    await expect(page.getByRole('columnheader', { name: field.label || field.name }).first()).toBeVisible()
  }

  const relationResponse = await page.evaluate(async () => {
    const response = await fetch('/api/meta/entities/work_order/relations', {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ target_entity_id: 'pm_schedule', name: 'Maintenance schedule acceptance', relation_type: 'many_to_many', on_delete: 'cascade' })
    })
    return { status: response.status, body: await response.text() }
  })
  expect(relationResponse.status, relationResponse.body).toBe(200)
  const relation = JSON.parse(relationResponse.body) as { id: string }
  const linksResponse = await page.evaluate(async (relationId) => {
    const response = await fetch(`/api/meta/relations/${encodeURIComponent(relationId)}/links/demo-wo-1`, {
      method: 'PUT', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ target_doc_ids: ['demo-pm-1', 'demo-pm-2'] })
    })
    return { status: response.status, body: await response.text() }
  }, relation.id)
  expect(linksResponse.status, linksResponse.body).toBe(200)

  const relatedRow = page.getByRole('row').filter({ hasText: 'Fix water pump' }).first()
  await relatedRow.getByRole('button', { name: 'Edit' }).click()
  const relatedDialog = page.getByRole('dialog')
  await expect(relatedDialog.getByText('Related records', { exact: true })).toBeVisible()
  await expect(relatedDialog.getByText('Maintenance schedule acceptance', { exact: true })).toBeVisible()
  await expect(relatedDialog.getByText('Monthly pump inspection', { exact: true })).toBeVisible()
  await expect(relatedDialog.getByText('Quarterly fire drill', { exact: true })).toBeVisible()
  await relatedDialog.getByRole('button', { name: 'Close' }).click()

  await page.getByRole('button', { name: 'New record' }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible({ timeout: 15_000 })

  const titleField = entity.fields.find((field: { name: string }) => field.name === 'title')
  const statusField = entity.fields.find((field: { name: string }) => field.name === 'status')
  expect(titleField).toBeTruthy()
  expect(statusField).toBeTruthy()

  const titleInput = dialog.getByLabel(new RegExp(`${titleField.label || titleField.name}\\*?$`, 'i'))
  const title = `Metadata UI ${Date.now()}`
  await titleInput.fill(title)

  if (statusField) {
    const statusLabel = statusField.label || statusField.name
    const statusSelect = dialog.getByRole('combobox', { name: new RegExp(statusLabel, 'i') })
    if (await statusSelect.count()) {
      await statusSelect.click()
      const statusOption = page.getByRole('option').first()
      if (await statusOption.count()) await statusOption.click()
    }
  }

  const createResponsePromise = page.waitForResponse(response => response.url().includes('/api/documents') && response.request().method() === 'POST', { timeout: 15_000 })
  await dialog.getByRole('button', { name: 'Save', exact: true }).click()
  const createResponse = await createResponsePromise
  expect(createResponse.status(), await createResponse.text()).toBe(200)
  const createdRecords = await page.evaluate(async () => {
    const response = await fetch('/api/documents?entity_id=work_order&limit=50&offset=0')
    if (!response.ok) throw new Error(`document list failed: ${response.status}`)
    return response.json()
  })
  expect(createdRecords.items.some((item: { payload: Record<string, unknown> }) => item.payload.title === title)).toBe(true)
  await page.reload()
  await expect(page.getByText(title, { exact: true })).toBeVisible({ timeout: 15_000 })

  // Open the metadata-generated row action and verify the same generic form
  // can update the record through the normal document API.
  const row = page.getByRole('row').filter({ hasText: title }).first()
  await row.getByRole('button', { name: 'Edit' }).click()
  const editDialog = page.getByRole('dialog')
  await expect(editDialog).toBeVisible({ timeout: 15_000 })
  const updatedTitle = `${title} updated`
  await editDialog.getByLabel(new RegExp(`${titleField.label || titleField.name}\*?$`, 'i')).fill(updatedTitle)
  await editDialog.getByRole('button', { name: 'Save', exact: true }).click()
  await expect(page.getByText(updatedTitle, { exact: true })).toBeVisible({ timeout: 15_000 })

  // Phase 5: generic record-level actions (duplicate) must be metadata-driven
  // and work on any entity through the same detail form, without an
  // entity-specific implementation.
  const updatedRow = page.getByRole('row').filter({ hasText: updatedTitle }).first()
  await updatedRow.getByRole('button', { name: 'Edit' }).click()
  const detailDialog = page.getByRole('dialog')
  await expect(detailDialog).toBeVisible({ timeout: 15_000 })
  await detailDialog.getByRole('button', { name: 'Duplicate' }).click()
  const duplicateTitleInput = detailDialog.getByLabel(new RegExp(`${titleField.label || titleField.name}\\*?$`, 'i'))
  await expect(duplicateTitleInput).toHaveValue(updatedTitle)
  const duplicatedTitle = `${updatedTitle} copy`
  await duplicateTitleInput.fill(duplicatedTitle)
  const duplicateCreatePromise = page.waitForResponse(response => response.url().includes('/api/documents') && response.request().method() === 'POST', { timeout: 15_000 })
  await detailDialog.getByRole('button', { name: 'Save', exact: true }).click()
  const duplicateCreateResponse = await duplicateCreatePromise
  expect(duplicateCreateResponse.status(), await duplicateCreateResponse.text()).toBe(200)
  await expect(page.getByText(duplicatedTitle, { exact: true })).toBeVisible({ timeout: 15_000 })
  await expect(page.getByText(updatedTitle, { exact: true })).toBeVisible({ timeout: 15_000 })
})
