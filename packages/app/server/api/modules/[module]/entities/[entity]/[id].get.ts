import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'module') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const id = getRouterParam(event, 'id') || ''
  return coreClient(event).getModuleDocument(module, entity, id)
})
