import { expect, test, type Page } from '@playwright/test'

async function login(page: Page, username: string, password: string) {
  // Log in through the API: UAuthForm's vee-validate state does not pick up
  // Playwright-filled values (submit sees empty fields), so driving the UI
  // form is flaky. The cookie set here exercises the real auth flow.
  // On a cold dev server the first navigations can abort (ERR_ABORTED) while
  // Vite optimizes deps and reloads, so retry the whole login flow.
  await expect(async () => {
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
  }).toPass({ timeout: 90_000 })
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

  // Delete the record. The confirm modal animates in; wait for it to
  // stabilize before clicking (headless "element is not stable" flake).
  await dialog.getByRole('button', { name: 'Delete' }).click()
  const confirmDelete = page.getByRole('dialog').getByRole('button', { name: 'Delete', exact: true })
  await confirmDelete.waitFor({ timeout: 15_000 })
  await page.waitForTimeout(500)
  await confirmDelete.click()
  await expect(page.getByText('Record deleted', { exact: true })).toBeVisible()
  await expect(page.getByText(updatedTitle)).toHaveCount(0)
})

test('search filters the work order list', async ({ page }) => {
  await login(page, 'demo', 'demo1234')
  await page.goto('/app/work_order')
  await expect(page.getByText('Fix water pump')).toBeVisible({ timeout: 15_000 })

  const searchInput = page.getByPlaceholder('Search…')
  // CRITICAL: Set up the response listener BEFORE typing, because Nuxt's useFetch
  // retains the old data in data.value while a background re-fetch is pending
  // (status: 'pending'). If we wait for the response AFTER typing, the old list
  // is still visible and toHaveCount(0) will see stale data before the new response
  // arrives. By waiting for the response FIRST, we guarantee the network request
  // has completed and documents.value has been updated before any DOM assertions.
  // On a cold dev server the page may still be hydrating when we start typing,
  // so keystrokes can be dropped (input stays empty, no request fires). Retry
  // the type+submit until the filtered request is actually observed.
  await expect(async () => {
    const responsePromise = page.waitForResponse(
      resp => resp.url().includes('/api/documents') && resp.url().includes('search=conveyor'),
      { timeout: 10_000 }
    )
    await searchInput.click()
    await page.keyboard.press('ControlOrMeta+a')
    await page.keyboard.press('Backspace')
    await page.keyboard.type('conveyor', { delay: 50 })
    await searchInput.press('Enter')
    await responsePromise
  }).toPass({ timeout: 90_000 })
  await page.waitForLoadState('networkidle')

  // Now safe to assert: Replace conveyor belt should be visible (it's in the filtered list)
  // and Fix water pump should have count 0 (it's been filtered out)
  await expect(page.getByRole('cell', { name: 'Replace conveyor belt' })).toBeVisible({ timeout: 15_000 })
  await expect(page.getByRole('cell', { name: 'Fix water pump' })).toHaveCount(0, { timeout: 15_000 })
})
