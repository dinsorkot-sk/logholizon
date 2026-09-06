import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<{ action?: unknown; expected_updated_at?: unknown }>(event)
  if (!id?.trim() || typeof body?.action !== 'string' || !body.action.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id and action are required' })
  }
  const expected = typeof body.expected_updated_at === 'string' && body.expected_updated_at.trim()
    ? body.expected_updated_at.trim()
    : undefined
  return coreClient(event).transitionDocument(id, body.action, expected)
})
