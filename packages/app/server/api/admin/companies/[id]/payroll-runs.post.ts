import { coreClient } from '../../../../core/client'

type CreatePayrollBody = { period: string; entry_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreatePayrollBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.period?.trim() || !body?.entry_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'period and entry_date are required' })
  }
  return coreClient(event).createPayrollRun(id, { period: body.period.trim(), entry_date: body.entry_date.trim() })
})
