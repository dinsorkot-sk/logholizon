import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  await coreClient().deleteWorkflowTransition(id)
  return { success: true }
})
