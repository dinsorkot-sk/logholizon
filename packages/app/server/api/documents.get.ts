import { coreClient } from '../core/client'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const entityId = typeof query.entity_id === 'string' ? query.entity_id : ''
  if (!entityId.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'entity_id is required' })
  }
  const limit = parseBoundedInt(query.limit, 50, 1, 100)
  const offset = parseBoundedInt(query.offset, 0, 0, Number.MAX_SAFE_INTEGER)
  return coreClient(event).listDocuments(entityId, limit, offset, {
    search: typeof query.search === 'string' ? query.search : undefined,
    status: typeof query.status === 'string' ? query.status : undefined,
    sortBy: typeof query.sort_by === 'string' ? query.sort_by : undefined,
    sortDir: typeof query.sort_dir === 'string' ? query.sort_dir : undefined,
    viewId: typeof query.view_id === 'string' ? query.view_id : undefined
  })
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