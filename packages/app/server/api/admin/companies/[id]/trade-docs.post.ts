import { coreClient } from '../../../../core/client'

type CreateTradeBody = { kind: string; doc_type: string; status?: string; partner: string; currency: string; entry_date: string; source_id?: string; lines?: { product_id?: string; description: string; qty: number; uom_id?: string; unit_price: number; tax_rule_id?: string }[] }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateTradeBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.kind?.trim() || !body?.doc_type?.trim() || !body?.partner?.trim() || !body?.currency?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'kind, doc_type, partner, currency, and entry_date are required' })
  }
  if (body.doc_type.trim() !== 'lead' && (!Array.isArray(body?.lines) || !body.lines.length)) {
    throw createError({ statusCode: 400, statusMessage: 'at least 1 line is required' })
  }
  return coreClient(event).createTradeDoc(id, {
    kind: body.kind.trim(),
    doc_type: body.doc_type.trim(),
    status: body.status?.trim() || undefined,
    partner: body.partner.trim(),
    currency: body.currency.trim(),
    entry_date: body.entry_date.trim(),
    source_id: body.source_id?.trim() || undefined,
    lines: body.lines || []
  })
})
