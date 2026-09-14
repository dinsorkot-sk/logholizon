import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const query = getQuery(event)
  const limit = Number(query.limit ?? 50)
  const offset = Number(query.offset ?? 0)
  return coreClient(event).listWorkflowHistory(id, limit, offset)
})
