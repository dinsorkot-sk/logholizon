import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'module') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const body = await readBody<{ id?: unknown; payload?: unknown }>(event)
  return coreClient(event).createModuleDocument(module, entity, String(body?.id || ''), (body?.payload || {}) as Record<string, unknown>)
})
