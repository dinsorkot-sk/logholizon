import { coreClient } from '../../../../core/client'

type ApplyMoveBody = { product_id: string; warehouse_id: string; move_type: string; qty: number; uom_id?: string; unit_cost?: number; entry_date: string; move_doc_id?: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<ApplyMoveBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.product_id?.trim() || !body?.warehouse_id?.trim() || !body?.move_type?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'product_id, warehouse_id, move_type, and entry_date are required' })
  }
  if (!Number.isFinite(body?.qty) || (body.qty as number) <= 0) {
    throw createError({ statusCode: 400, statusMessage: 'qty must be > 0' })
  }
  return coreClient(event).applyStockMove(id, {
    product_id: body.product_id.trim(),
    warehouse_id: body.warehouse_id.trim(),
    move_type: body.move_type.trim(),
    qty: body.qty,
    uom_id: body.uom_id?.trim() || undefined,
    unit_cost: body.unit_cost ?? 0,
    entry_date: body.entry_date.trim(),
    move_doc_id: body.move_doc_id?.trim() || undefined
  })
})
