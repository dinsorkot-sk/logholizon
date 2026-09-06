import { coreClient } from '../../../../core/client'

type CreateViewBody = { name: string; config?: Record<string, unknown> }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateViewBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  return coreClient(event).createEntityView(id, {
    name: body.name,
    config: body.config || {}
  })
})