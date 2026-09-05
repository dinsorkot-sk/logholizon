import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const query = getQuery(event)
  const limit = parseBoundedInt(query.limit, 50, 1, 100)
  const offset = parseBoundedInt(query.offset, 0, 0, Number.MAX_SAFE_INTEGER)
  return coreClient().listDocumentAudit(id, limit, offset)
})

function parseBoundedInt(
  raw: unknown,
  fallback: number,
  min: number,
  max: number
): number {
  if (raw === undefined) return fallback
  const value = Number(raw)
  if (!Number.isInteger(value) || value < min || value > max) {
    throw createError({
      statusCode: 400,
      statusMessage: `expected integer in [${min}, ${max}]`
    })
  }
  return value
}