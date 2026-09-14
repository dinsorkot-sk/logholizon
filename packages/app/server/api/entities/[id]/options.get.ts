import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const query = getQuery(event)
  const search = typeof query.search === 'string' ? query.search : undefined
  const limit = typeof query.limit === 'string' ? Number(query.limit) : undefined
  if (limit !== undefined && (!Number.isInteger(limit) || limit < 1 || limit > 100)) {
    throw createError({ statusCode: 400, statusMessage: 'limit must be between 1 and 100' })
  }
  return coreClient(event).getEntityOptions(id, { search, limit })
})
