import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const query = getQuery(event)
  const documentId = typeof query.document_id === 'string' ? query.document_id : undefined
  return coreClient(event).listEvents(id, documentId)
})
