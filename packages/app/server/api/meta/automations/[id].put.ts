import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody(event)
  return coreClient(event).updateAutomation(id, {
    condition: body?.condition,
    schedule: body?.schedule,
    actions: body?.actions,
    max_attempts: body?.max_attempts,
    active: body?.active
  })
})
