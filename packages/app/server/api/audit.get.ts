import { coreClient } from '../core/client'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const limit = parseBoundedInt(query.limit, 50, 1, 100)
  const offset = parseBoundedInt(query.offset, 0, 0, Number.MAX_SAFE_INTEGER)
  return coreClient(event).listGlobalAudit(limit, offset, {
    entityId: typeof query.entity_id === 'string' ? query.entity_id : undefined,
    action: typeof query.action === 'string' ? query.action : undefined,
    search: typeof query.search === 'string' ? query.search : undefined
  })
})

function parseBoundedInt(
  raw: unknown,
  fallback: number,
  min: number,
  max: number
): number {
  if (raw === undefined) return fallback
  const value = typeof raw === 'string' ? Number.parseInt(raw, 10) : Number(raw)
  if (!Number.isFinite(value)) return fallback
  return Math.min(max, Math.max(min, Math.trunc(value)))
}
