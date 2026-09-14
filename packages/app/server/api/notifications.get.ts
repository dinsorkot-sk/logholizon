import { coreClient } from '../core/client'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const unread = query.unread === 'true'
  return coreClient(event).listMyNotifications(unread)
})
