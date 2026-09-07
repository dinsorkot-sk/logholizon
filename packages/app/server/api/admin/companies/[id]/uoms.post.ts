import { coreClient } from '../../../../core/client'

type CreateUomBody = { code: string; name: string; dimension: string; factor_to_base: number; is_base?: boolean }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateUomBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.code?.trim() || !body?.name?.trim() || !body?.dimension?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'code, name, and dimension are required' })
  }
  if (!Number.isFinite(body?.factor_to_base) || (body.factor_to_base as number) <= 0) {
    throw createError({ statusCode: 400, statusMessage: 'factor_to_base must be > 0' })
  }
  return coreClient(event).createUom(id, {
    code: body.code.trim(),
    name: body.name.trim(),
    dimension: body.dimension.trim(),
    factor_to_base: body.factor_to_base,
    is_base: body.is_base ?? false
  })
})
