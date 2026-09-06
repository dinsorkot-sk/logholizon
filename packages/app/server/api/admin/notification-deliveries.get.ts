import { coreClient } from '../../core/client'

export default defineEventHandler((event) => {
  const query = getQuery(event)
  const limit = Number(query.limit ?? 50)
  const offset = Number(query.offset ?? 0)
  if (!Number.isInteger(limit) || limit < 1 || limit > 100 || !Number.isInteger(offset) || offset < 0) {
    throw createError({ statusCode: 400, statusMessage: 'invalid pagination' })
  }
  return coreClient(event).listNotificationDeliveries(limit, offset)
})
