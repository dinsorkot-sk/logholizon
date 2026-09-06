import { coreClient } from '../../../../core/client'

type UpdateStateBody = { label: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateStateBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.label?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'label is required' })
  }
  return coreClient(event).updateWorkflowState(id, { label: body.label })
})
