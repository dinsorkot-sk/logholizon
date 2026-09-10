import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  await coreClient(event).markNotificationRead(id)
  return null
})
