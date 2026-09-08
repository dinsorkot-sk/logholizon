import { coreClient } from '../../../../core/client'

type AddPayslipBody = { employee_id: string; gross: number; deductions: number }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<AddPayslipBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.employee_id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'employee_id is required' })
  }
  if (!Number.isFinite(body?.gross) || (body.gross as number) < 0 || !Number.isFinite(body?.deductions) || (body.deductions as number) < 0) {
    throw createError({ statusCode: 400, statusMessage: 'gross and deductions must be >= 0' })
  }
  return coreClient(event).addPayslip(id, { employee_id: body.employee_id.trim(), gross: body.gross, deductions: body.deductions })
})
