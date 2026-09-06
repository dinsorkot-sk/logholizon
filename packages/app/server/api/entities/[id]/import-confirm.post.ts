import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const body = await readBody(event)
  if (typeof body !== 'string' || !body.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'CSV body is required' })
  }
  return coreClient(event).confirmImportForUser(id, body)
})
