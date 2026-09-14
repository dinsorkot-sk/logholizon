import { coreClient } from '../../../../core/client'

type UpdatePermissionsBody = {
  permissions: { role: string; can_view?: boolean; can_edit?: boolean }[]
}

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdatePermissionsBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!Array.isArray(body?.permissions)) {
    throw createError({ statusCode: 400, statusMessage: 'permissions must be an array' })
  }
  return coreClient(event).updateEntityPermissions(id, body.permissions.map(p => ({
    role: p.role,
    can_view: !!p.can_view,
    can_edit: !!p.can_edit
  })))
})