import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<{ body?: string }>(event)
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  if (!body?.body?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'body is required' })
  }
  if (body.body.trim().length > 2000) {
    throw createError({ statusCode: 400, statusMessage: 'body must be at most 2000 characters' })
  }
  return coreClient(event).createDocComment(id, body.body.trim())
})
