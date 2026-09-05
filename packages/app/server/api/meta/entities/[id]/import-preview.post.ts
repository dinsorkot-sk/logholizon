import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readRawBody(event, 'utf8')
  if (!id.trim() || !body?.trim()) throw createError({ statusCode: 400, statusMessage: 'entity ID and CSV are required' })
  return coreClient().previewImport(id, body)
})
