import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  return coreClient(event).getDashboardForUser(id)
})
