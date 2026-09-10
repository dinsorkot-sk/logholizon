import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  return coreClient(event).installModulePackage((body?.package || {}) as Record<string, unknown>)
})
