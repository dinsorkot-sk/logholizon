import { coreClient } from '../../../../core/client'

type SetFxRateBody = { from_currency: string; to_currency: string; rate: number; rate_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<SetFxRateBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.from_currency?.trim() || !body?.to_currency?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'from_currency and to_currency are required' })
  }
  if (typeof body?.rate !== 'number' || !(body.rate > 0)) {
    throw createError({ statusCode: 400, statusMessage: 'rate must be a positive number' })
  }
  if (!body?.rate_date?.trim()) throw createError({ statusCode: 400, statusMessage: 'rate_date is required' })
  return coreClient(event).setFxRate(id, {
    from_currency: body.from_currency.trim(),
    to_currency: body.to_currency.trim(),
    rate: body.rate,
    rate_date: body.rate_date.trim()
  })
})
