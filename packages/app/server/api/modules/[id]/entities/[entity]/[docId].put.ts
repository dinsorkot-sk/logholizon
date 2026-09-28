import { coreClient } from '../../../../../core/client'

export default defineEventHandler(async (event) => {
  const module = getRouterParam(event, 'id') || ''
  const entity = getRouterParam(event, 'entity') || ''
  const id = getRouterParam(event, 'docId') || ''
  const body = await readBody<{ payload?: unknown; expected_updated_at?: unknown }>(event)
  const expected = typeof body?.expected_updated_at === 'string' ? body.expected_updated_at : undefined
  return coreClient(event).updateModuleDocument(module, entity, id, (body?.payload || {}) as Record<string, unknown>, expected)
})
