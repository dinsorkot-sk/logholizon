import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  return coreClient(event).listObservabilityLogs(query as Record<string, unknown>)
})
