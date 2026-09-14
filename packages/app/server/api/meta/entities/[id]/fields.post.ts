import { coreClient } from '../../../../core/client'

type CreateFieldBody = { name: string; type: string; required?: boolean; is_status?: boolean; ref_entity?: string | null; computed_expr?: string | null }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateFieldBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim() || !body?.type?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'name and type are required' })
  }
  return coreClient(event).createField(id, {
    name: body.name,
    type: body.type,
    required: !!body.required,
    is_status: !!body.is_status,
    ref_entity: body.ref_entity ?? null,
    computed_expr: body.computed_expr ?? null
  })
})