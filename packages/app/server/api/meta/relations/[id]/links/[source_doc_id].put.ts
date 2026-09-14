import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const sourceDocId = getRouterParam(event, 'source_doc_id') || ''
  const body = await readBody<{ target_doc_ids?: unknown }>(event)
  const ids = Array.isArray(body?.target_doc_ids) ? (body.target_doc_ids as unknown[]).map(String) : []
  return coreClient(event).setRelationLinks(id, sourceDocId, ids)
})
