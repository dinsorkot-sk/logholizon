import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'module') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody<{ payload?: unknown; expected_updated_at?: unknown }>(event)
  const expected = typeof body?.expected_updated_at === 'string' ? body.expected_updated_at : undefined
  return coreClient(event).updateModuleDocument(module, entity, id, (body?.payload || {}) as Record<string, unknown>, expected)
})
