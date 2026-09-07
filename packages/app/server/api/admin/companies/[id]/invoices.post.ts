import { coreClient } from '../../../../core/client'

type CreateInvoiceBody = { kind: string; partner: string; currency: string; entry_date: string; lines: { description: string; quantity: number; unit_price: number; tax_rule_id?: string }[] }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateInvoiceBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.kind?.trim() || !body?.partner?.trim() || !body?.currency?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'kind, partner, currency, and entry_date are required' })
  }
  if (!Array.isArray(body?.lines) || !body.lines.length) {
    throw createError({ statusCode: 400, statusMessage: 'at least 1 line is required' })
  }
  return coreClient(event).createInvoice(id, {
    kind: body.kind.trim(),
    partner: body.partner.trim(),
    currency: body.currency.trim(),
    entry_date: body.entry_date.trim(),
    lines: body.lines
  })
})
