import { coreClient } from '../../../../core/client'

type CreateAccountBody = { code: string; name: string; type: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateAccountBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.code?.trim()) throw createError({ statusCode: 400, statusMessage: 'code is required' })
  if (!body?.name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  if (!body?.type?.trim()) throw createError({ statusCode: 400, statusMessage: 'type is required' })
  return coreClient(event).createGlAccount(id, { code: body.code.trim(), name: body.name.trim(), type: body.type.trim() })
})
