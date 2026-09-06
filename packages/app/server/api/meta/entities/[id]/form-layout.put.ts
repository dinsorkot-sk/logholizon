import { coreClient } from '../../../../core/client'

type UpdateLayoutBody = { config?: Record<string, unknown> }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateLayoutBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  return coreClient(event).updateFormLayout(id, body?.config || {})
})
