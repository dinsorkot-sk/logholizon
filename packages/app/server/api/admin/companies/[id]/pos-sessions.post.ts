import { coreClient } from '../../../../core/client'

type OpenSessionBody = { name: string; warehouse_id: string; opening_cash: number; entry_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<OpenSessionBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim() || !body?.warehouse_id?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'name, warehouse_id, and entry_date are required' })
  }
  if (!Number.isFinite(body?.opening_cash) || (body.opening_cash as number) < 0) {
    throw createError({ statusCode: 400, statusMessage: 'opening_cash must be >= 0' })
  }
  return coreClient(event).openPosSession(id, {
    name: body.name.trim(),
    warehouse_id: body.warehouse_id.trim(),
    opening_cash: body.opening_cash,
    entry_date: body.entry_date.trim()
  })
})
