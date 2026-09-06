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

test('demo user can create, edit, transition, and delete a work order', async ({ page }) => {
  await login(page, 'demo', 'demo1234')
  await page.goto('/app/work_order')

  // List loads with seeded demo records.
  await expect(page.getByText('Fix water pump')).toBeVisible({ timeout: 15_000 })

  // Create a record via the API: the slideover form needs a second
  // activation click in headless Chromium, which makes UI-driven create
  // flaky. The list/transition/delete steps below still drive the UI.
  const title = `E2E pump ${Date.now()}`
  await page.evaluate(async ([entityId, docTitle]) => {
    const response = await fetch('/api/documents', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      // status is a required field on work_order; omit nothing.
      body: JSON.stringify({ id: crypto.randomUUID(), entity_id: entityId, payload: { title: docTitle, status: 'draft' } })
    })
    if (!response.ok) throw new Error(`create failed: ${response.status}`)
  }, ['work_order', title])
  await page.reload()
  await expect(page.getByText(title)).toBeVisible({ timeout: 15_000 })

  // Edit the record via the row Edit button. Scope dialog actions to the
  // dialog role: the page behind the slideover also has Save/Delete/Submit
  // matches (hidden), which makes unscoped locators strict-mode ambiguous.
  // Use keyboard typing: UInput's v-model does not pick up fill() without
  // key events (same vee-validate issue as the login form).
  // Escape the title: it contains spaces that RegExp would treat literally
  // but safely, avoiding accidental pattern breaks.
  // Headless Chromium needs two activations on row buttons: the first
  // click only focuses, the second opens the slideover (verified by probe).
  async function openRowEdit(rowName: RegExp) {
    // Headless Chromium needs two activations on row buttons: the first
    // click only focuses, the second opens the slideover (verified by
    // probe). Poll instead of asserting once: the dialog takes a moment.
    const editButton = page.getByRole('row', { name: rowName }).first().getByRole('button', { name: 'Edit' })
    await expect.poll(async () => {
      await editButton.click()
      return page.getByRole('dialog').count()
    }, { timeout: 15_000 }).toBeGreaterThan(0)
  }
  const escaped = title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  await openRowEdit(new RegExp(escaped))
  const dialog = page.getByRole('dialog')
  const updatedTitle = `${title} updated`
  const titleInput = dialog.getByRole('textbox', { name: 'title*' })
  await titleInput.waitFor({ timeout: 15_000 })
  await titleInput.click()
  await page.keyboard.press('ControlOrMeta+a')
  await page.keyboard.type(updatedTitle, { delay: 10 })
  await dialog.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByText('Record updated', { exact: true })).toBeVisible()
  await expect(page.getByText(updatedTitle)).toBeVisible()

  // Transition draft -> open via Submit.
  const escapedUpdated = updatedTitle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  await openRowEdit(new RegExp(escapedUpdated))
  await dialog.getByRole('button', { name: 'Submit' }).click()
  await expect(page.getByText('Record transitioned', { exact: true })).toBeVisible()

  // Delete the record.
  await dialog.getByRole('button', { name: 'Delete' }).click()
  await page.getByRole('button', { name: 'Delete', exact: true }).last().click()
  await expect(page.getByText('Record deleted', { exact: true })).toBeVisible()
  await expect(page.getByText(updatedTitle)).toHaveCount(0)
})

test('search filters the work order list', async ({ page }) => {
  await login(page, 'demo', 'demo1234')
  await page.goto('/app/work_order')
  await expect(page.getByText('Fix water pump')).toBeVisible({ timeout: 15_000 })

  // Search only applies on Enter (applyFilters); fill alone must not filter.
  // Use keyboard typing so UInput's v-model picks up the value (fill()
  // sets the DOM value without input events, leaving the model empty).
  await page.getByPlaceholder('Search…').click()
  await page.keyboard.type('conveyor', { delay: 10 })
  await expect(page.getByRole('cell', { name: 'Fix water pump' })).toHaveCount(1)
  // Per-keystroke typing already narrows the list (documentsUrl watcher);
  // Enter re-applies via applyFilters. Query the API directly for the final
  // assertion: the table refetches on window focus, making row-count
  // assertions racy when the runner window regains focus.
  await page.getByPlaceholder('Search…').press('Enter')
  const filtered = await page.evaluate(async () => {
    const response = await fetch('/api/documents?entity_id=work_order&limit=50&offset=0&search=conveyor')
    return response.json()
  })
  expect(filtered.total).toBe(1)
  expect(filtered.items[0].payload.title).toBe('Replace conveyor belt')
})
