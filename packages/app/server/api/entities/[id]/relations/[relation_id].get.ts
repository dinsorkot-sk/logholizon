import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const relationId = getRouterParam(event, 'relation_id') || ''
  return coreClient(event).relatedDocuments(id, relationId)
})
