import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const body = await readBody<{ permissions?: unknown }>(event)
  if (!Array.isArray(body?.permissions)) {
    throw createError({ statusCode: 400, statusMessage: 'permissions must be an array' })
  }
  for (const entry of body.permissions as Record<string, unknown>[]) {
    if (typeof entry?.field_id !== 'string' || !entry.field_id.trim()
      || typeof entry?.role !== 'string' || !entry.role.trim()
      || typeof entry?.can_view !== 'boolean' || typeof entry?.can_edit !== 'boolean') {
      throw createError({ statusCode: 400, statusMessage: 'each permission needs field_id, role, can_view, can_edit' })
    }
  }
  return coreClient(event).updateFieldPermissions(
    id,
    body.permissions as { field_id: string; role: string; can_view: boolean; can_edit: boolean }[]
  )
})
