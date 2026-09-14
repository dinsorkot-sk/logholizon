import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const actionId = getRouterParam(event, 'action_id') || ''
  const body = await readBody(event)
  if (!id.trim() || !actionId.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'entity and action are required' })
  }
  return coreClient(event).executeModuleAction(id, actionId, {
    document_id: body?.document_id,
    payload: body?.payload,
    expected_updated_at: body?.expected_updated_at
  })
})
