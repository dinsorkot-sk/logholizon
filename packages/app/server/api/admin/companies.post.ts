import { coreClient } from '../../core/client'

type CreateCompanyBody = { name: string; base_currency: string }

export default defineEventHandler(async (event) => {
  const body = await readBody<CreateCompanyBody>(event)
  if (!body?.name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  if (!body?.base_currency?.trim()) throw createError({ statusCode: 400, statusMessage: 'base_currency is required' })
  return coreClient(event).createCompany({ name: body.name.trim(), base_currency: body.base_currency.trim() })
})
