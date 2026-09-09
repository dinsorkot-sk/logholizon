import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  return coreClient(event).createDashboard(body)
})

