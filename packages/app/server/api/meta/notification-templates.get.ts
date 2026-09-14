import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  return coreClient(event).listNotificationTemplates()
})
