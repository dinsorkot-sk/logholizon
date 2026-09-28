import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'id') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const id = getRouterParam(event, 'docId') || ''
  return coreClient(event).getModuleDocument(module, entity, id)
})
