import { coreClient } from '../../../../core/client'

type PostEntryBody = { memo?: string; entry_date: string; lines: { account_id: string; debit?: number; credit?: number; memo?: string }[] }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<PostEntryBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.entry_date?.trim()) throw createError({ statusCode: 400, statusMessage: 'entry_date is required' })
  if (!Array.isArray(body?.lines) || body.lines.length < 2) {
    throw createError({ statusCode: 400, statusMessage: 'at least 2 lines are required' })
  }
  return coreClient(event).postJournalEntry(id, {
    memo: body.memo || '',
    entry_date: body.entry_date.trim(),
    lines: body.lines
  })
})
