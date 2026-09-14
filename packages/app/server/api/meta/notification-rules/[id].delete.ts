import { coreClient } from '../../../core/client'

export default defineEventHandler((event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  return coreClient(event).deleteNotificationRule(id)
})
