import { coreClient } from '../../../../core/client'

type CreateMfgOrderBody = { bom_id: string; qty: number; warehouse_id: string; entry_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateMfgOrderBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.bom_id?.trim() || !body?.warehouse_id?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'bom_id, warehouse_id, and entry_date are required' })
  }
  if (!Number.isFinite(body?.qty) || (body.qty as number) <= 0) {
    throw createError({ statusCode: 400, statusMessage: 'qty must be > 0' })
  }
  return coreClient(event).createMfgOrder(id, {
    bom_id: body.bom_id.trim(),
    qty: body.qty,
    warehouse_id: body.warehouse_id.trim(),
    entry_date: body.entry_date.trim()
  })
})
