import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'entity ID is required' })
  const csv = await coreClient(event).exportDocuments(id)
  setHeader(event, 'content-type', 'text/csv; charset=utf-8')
  setHeader(event, 'content-disposition', `attachment; filename="${id}.csv"`)
  return csv
})
