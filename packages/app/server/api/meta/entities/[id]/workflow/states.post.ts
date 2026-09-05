import { coreClient } from '../../../../../core/client'

type CreateStateBody = { name: string; label: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateStateBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim() || !body?.label?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'name and label are required' })
  }
  return coreClient().createWorkflowState(id, { name: body.name, label: body.label })
})
