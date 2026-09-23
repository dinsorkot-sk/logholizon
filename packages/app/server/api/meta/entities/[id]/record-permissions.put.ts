import { coreClient } from '../../../../core/client'

type UpdateRecordPermissionsBody = {
  permissions: { entity_id?: string; role: string; scope: string; owner_field?: string | null }[]
}

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateRecordPermissionsBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!Array.isArray(body?.permissions)) {
    throw createError({ statusCode: 400, statusMessage: 'permissions must be an array' })
  }
  return coreClient(event).updateRecordPermissions(id, body.permissions.map(p => ({
    entity_id: id,
    role: p.role,
    scope: p.scope,
    owner_field: p.owner_field || null
  })))
})
