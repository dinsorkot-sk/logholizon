import { coreClient } from '../../../../core/client'

type CreateFieldBody = { name: string; type: string; required?: boolean; is_status?: boolean }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateFieldBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim() || !body?.type?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'name and type are required' })
  }
  return coreClient(event).createField(id, {
    name: body.name,
    type: body.type,
    required: !!body.required,
    is_status: !!body.is_status
  })
})