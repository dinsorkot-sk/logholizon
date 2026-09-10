import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'module') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const body = await readBody<{ ids?: unknown }>(event)
  const ids = Array.isArray(body?.ids) ? (body.ids as unknown[]).map(String) : []
  return coreClient(event).bulkDeleteModuleDocuments(module, entity, ids)
})
