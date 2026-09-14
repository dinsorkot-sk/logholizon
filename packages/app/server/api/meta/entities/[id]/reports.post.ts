import { coreClient } from '../../../../core/client'

type CreateReportBody = { name: string; config?: Record<string, unknown> }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateReportBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  return coreClient(event).createReport(id, {
    name: body.name,
    config: body.config || {}
  })
})
