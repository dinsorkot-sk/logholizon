import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'id') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const id = getRouterParam(event, 'docId') || ''
  await coreClient(event).deleteModuleDocument(module, entity, id)
  return null
})
