import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const body = await readBody<{ payload?: unknown }>(event)
  if (typeof body?.payload !== 'object' || body.payload === null || Array.isArray(body.payload)) {
    throw createError({ statusCode: 400, statusMessage: 'payload must be a JSON object' })
  }
  return coreClient().updateDocument(id, body.payload as Record<string, unknown>)
})