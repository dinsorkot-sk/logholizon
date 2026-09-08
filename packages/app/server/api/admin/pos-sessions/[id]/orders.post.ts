import { coreClient } from '../../../../core/client'

type CreatePosOrderBody = { partner?: string; currency: string; tendered: number; lines: { product_id: string; description: string; qty: number; uom_id?: string; unit_price: number; tax_rule_id?: string }[] }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreatePosOrderBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.currency?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'currency is required' })
  }
  if (!Number.isFinite(body?.tendered) || (body.tendered as number) < 0) {
    throw createError({ statusCode: 400, statusMessage: 'tendered must be >= 0' })
  }
  if (!Array.isArray(body?.lines) || !body.lines.length) {
    throw createError({ statusCode: 400, statusMessage: 'at least 1 line is required' })
  }
  return coreClient(event).createPosOrder(id, {
    partner: body.partner?.trim() || undefined,
    currency: body.currency.trim(),
    tendered: body.tendered,
    lines: body.lines
  })
})
