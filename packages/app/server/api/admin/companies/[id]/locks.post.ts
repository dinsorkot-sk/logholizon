import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<{ period?: string }>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.period?.trim()) throw createError({ statusCode: 400, statusMessage: 'period is required' })
  return coreClient(event).lockPeriod(id, body.period.trim())
})
