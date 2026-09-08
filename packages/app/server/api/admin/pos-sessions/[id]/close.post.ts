import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<{ closing_cash: number }>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!Number.isFinite(body?.closing_cash) || (body.closing_cash as number) < 0) {
    throw createError({ statusCode: 400, statusMessage: 'closing_cash must be >= 0' })
  }
  return coreClient(event).closePosSession(id, body.closing_cash)
})
