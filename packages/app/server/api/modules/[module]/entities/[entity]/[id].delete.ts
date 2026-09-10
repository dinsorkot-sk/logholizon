import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'module') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const id = getRouterParam(event, 'id') || ''
  await coreClient(event).deleteModuleDocument(module, entity, id)
  return null
})
