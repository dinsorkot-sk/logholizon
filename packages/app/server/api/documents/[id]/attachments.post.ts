import { coreClient } from '../../../core/client'

const ALLOWED_TYPES = new Set([
  'application/pdf',
  'text/plain',
  'text/csv',
  'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
])

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const headers = getHeaders(event)
  const filename = String(headers['x-filename'] || '').trim()
  const contentType = String(headers['content-type'] || '').split(';')[0]?.trim().toLowerCase() || ''
  if (!filename) {
    throw createError({ statusCode: 400, statusMessage: 'x-filename is required' })
  }
  const isImage = contentType.startsWith('image/')
  if (!isImage && !ALLOWED_TYPES.has(contentType)) {
    throw createError({ statusCode: 400, statusMessage: 'unsupported file type' })
  }
  const raw = await readRawBody(event, false)
  const bytes = raw instanceof Uint8Array ? raw : new TextEncoder().encode(String(raw || ''))
  if (!bytes.length) {
    throw createError({ statusCode: 400, statusMessage: 'file is empty' })
  }
  if (bytes.length > 5 * 1024 * 1024) {
    throw createError({ statusCode: 400, statusMessage: 'file must be at most 5MB' })
  }
  return coreClient(event).uploadDocAttachment(id, { filename, contentType, data: bytes })
})
