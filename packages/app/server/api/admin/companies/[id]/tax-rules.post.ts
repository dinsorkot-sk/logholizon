import { coreClient } from '../../../../core/client'

type CreateTaxRuleBody = { name: string; rate: number; is_inclusive?: boolean; is_withholding?: boolean }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateTaxRuleBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  if (typeof body?.rate !== 'number') throw createError({ statusCode: 400, statusMessage: 'rate is required' })
  return coreClient(event).createTaxRule(id, {
    name: body.name.trim(),
    rate: body.rate,
    ...(body.is_inclusive !== undefined ? { is_inclusive: body.is_inclusive } : {}),
    ...(body.is_withholding !== undefined ? { is_withholding: body.is_withholding } : {})
  })
})
