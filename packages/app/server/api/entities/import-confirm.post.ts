import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const body = await readRawBody(event, false)
  if (!body || (body instanceof Uint8Array && body.length === 0)) {
    throw createError({ statusCode: 400, statusMessage: 'XLSX body is required' })
  }
  const bytes = body instanceof Uint8Array ? body : new TextEncoder().encode(body)
  return coreClient(event).confirmWorkbookImport(bytes)
})
