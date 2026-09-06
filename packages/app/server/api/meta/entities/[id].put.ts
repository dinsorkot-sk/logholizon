import { coreClient } from '../../../core/client'

type UpdateEntityBody = { name: string; label: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateEntityBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim() || !body?.label?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'name and label are required' })
  }
  return coreClient(event).updateEntity(id, { name: body.name, label: body.label })
})