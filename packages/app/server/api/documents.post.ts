import { coreClient, type CreateDocumentInput } from '../core/client'

export default defineEventHandler(async (event) => {
  const body = await readBody<CreateDocumentInput>(event)
  if (!body?.id?.trim() || !body?.entity_id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id and entity_id are required' })
  }
  if (typeof body.payload !== 'object' || body.payload === null || Array.isArray(body.payload)) {
    throw createError({ statusCode: 400, statusMessage: 'payload must be a JSON object' })
  }
  return coreClient(event).createDocument(body)
})