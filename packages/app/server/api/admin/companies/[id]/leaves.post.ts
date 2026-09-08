import { coreClient } from '../../../../core/client'

type RequestLeaveBody = { employee_id: string; kind: string; from_date: string; to_date: string }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<RequestLeaveBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.employee_id?.trim() || !body?.kind?.trim() || !body?.from_date?.trim() || !body?.to_date?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'employee_id, kind, from_date, and to_date are required' })
  }
  return coreClient(event).requestLeave(id, {
    employee_id: body.employee_id.trim(),
    kind: body.kind.trim(),
    from_date: body.from_date.trim(),
    to_date: body.to_date.trim()
  })
})
