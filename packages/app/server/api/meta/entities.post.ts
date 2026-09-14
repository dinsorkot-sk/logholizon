import { coreClient } from '../../core/client'

type CreateEntityBody = { id: string; name: string; label: string }

export default defineEventHandler(async (event) => {
  const body = await readBody<CreateEntityBody>(event)
  if (!body?.id?.trim() || !body?.name?.trim() || !body?.label?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id, name, and label are required' })
  }
  return coreClient(event).createEntity(body)
})
