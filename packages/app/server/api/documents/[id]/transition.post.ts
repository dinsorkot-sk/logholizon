import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<{ action?: unknown }>(event)
  if (!id?.trim() || typeof body?.action !== 'string' || !body.action.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id and action are required' })
  }
  return coreClient().transitionDocument(id, body.action)
})
