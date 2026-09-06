import { coreClient } from '../../../../../core/client'

type CreateTransitionBody = { from_state: string; to_state: string; action: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateTransitionBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.from_state?.trim() || !body?.to_state?.trim() || !body?.action?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'from_state, to_state, and action are required' })
  }
  return coreClient(event).createWorkflowTransition(id, {
    from_state: body.from_state,
    to_state: body.to_state,
    action: body.action
  })
})
