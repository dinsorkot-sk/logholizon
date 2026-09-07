import { coreClient } from '../../../../core/client'

type CreatePaymentBody = { kind: string; partner: string; currency: string; amount: number; entry_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreatePaymentBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.kind?.trim() || !body?.partner?.trim() || !body?.currency?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'kind, partner, currency, and entry_date are required' })
  }
  if (typeof body?.amount !== 'number') throw createError({ statusCode: 400, statusMessage: 'amount is required' })
  return coreClient(event).createPayment(id, {
    kind: body.kind.trim(),
    partner: body.partner.trim(),
    currency: body.currency.trim(),
    amount: body.amount,
    entry_date: body.entry_date.trim()
  })
})
