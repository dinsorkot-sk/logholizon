import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const sourceDocId = getRouterParam(event, 'source_doc_id') || ''
  return coreClient(event).listRelationLinks(id, sourceDocId)
})
