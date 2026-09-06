import { coreClient } from '../../core/client'

export default defineEventHandler((event) => {
  const query = getQuery(event)
  const entityId = typeof query.entity_id === 'string' ? query.entity_id : ''
  const groupBy = typeof query.group_by === 'string' ? query.group_by : ''
  if (!entityId.trim() || !groupBy.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'entity_id and group_by are required' })
  }
  return coreClient(event).getReportAggregate(entityId, groupBy)
})
