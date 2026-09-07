import { coreClient } from '../../core/client'

type CreateCurrencyBody = { code: string; name: string; decimals?: number }

export default defineEventHandler(async (event) => {
  const body = await readBody<CreateCurrencyBody>(event)
  if (!body?.code?.trim()) throw createError({ statusCode: 400, statusMessage: 'code is required' })
  if (!body?.name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  return coreClient(event).createCurrency({
    code: body.code.trim(),
    name: body.name.trim(),
    ...(body.decimals !== undefined ? { decimals: body.decimals } : {})
  })
})
