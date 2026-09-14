import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody(event)
  const version = Number(body?.version)
  if (!Number.isInteger(version) || version < 1) {
    throw createError({ statusCode: 400, statusMessage: 'version must be >= 1' })
  }
  return coreClient(event).rollbackModule(id, version)
})
