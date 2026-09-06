import { coreClient } from '../../core/client'

export default defineEventHandler((event) => {
  const query = getQuery(event)
  const entityId = typeof query.entity_id === 'string' ? query.entity_id : ''
  if (!entityId.trim()) throw createError({ statusCode: 400, statusMessage: 'entity_id is required' })
  return coreClient().getPmSummary(entityId)
})