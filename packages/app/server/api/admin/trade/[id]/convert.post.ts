import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<{ entry_date?: string }>(event).catch(() => ({} as { entry_date?: string }))
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  return coreClient(event).convertTradeDoc(id, body?.entry_date?.trim() ? { entry_date: body.entry_date.trim() } : undefined)
})
