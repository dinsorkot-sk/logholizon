import { coreClient } from '../../../../core/client'

type CreateEmployeeBody = { code: string; name: string; base_salary: number; currency: string; hire_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateEmployeeBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.code?.trim() || !body?.name?.trim() || !body?.currency?.trim() || !body?.hire_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'code, name, currency, and hire_date are required' })
  }
  if (!Number.isFinite(body?.base_salary) || (body.base_salary as number) < 0) {
    throw createError({ statusCode: 400, statusMessage: 'base_salary must be >= 0' })
  }
  return coreClient(event).createEmployee(id, {
    code: body.code.trim(),
    name: body.name.trim(),
    base_salary: body.base_salary,
    currency: body.currency.trim(),
    hire_date: body.hire_date.trim()
  })
})
