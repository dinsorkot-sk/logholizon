import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<{ invoice_id?: string; amount?: number }>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.invoice_id?.trim()) throw createError({ statusCode: 400, statusMessage: 'invoice_id is required' })
  if (typeof body?.amount !== 'number') throw createError({ statusCode: 400, statusMessage: 'amount is required' })
  return coreClient(event).allocatePayment(id, { invoice_id: body.invoice_id.trim(), amount: body.amount })
})
