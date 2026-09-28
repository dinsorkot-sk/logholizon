import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'id') || ''
  const entity = getRouterParam(event, 'entity') || ''
  return coreClient(event).listModuleDocuments(module, entity, getQuery(event) as Record<string, unknown>)
})
