import { coreClient } from '../../../core/client'

type UpdateUserBody = { role: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateUserBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.role?.trim()) throw createError({ statusCode: 400, statusMessage: 'role is required' })
  return coreClient(event).updateUser(id, body.role)
})