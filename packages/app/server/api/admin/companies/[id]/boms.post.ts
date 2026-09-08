import { coreClient } from '../../../../core/client'

type CreateBomBody = { product_id: string; name: string; lines: { component_id: string; qty: number; uom_id?: string }[] }

export default defineEventHandler(async (event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const body = await readBody<CreateBomBody>(event)
  if (!id.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.product_id?.trim() || !body?.name?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'product_id and name are required' })
  }
  if (!Array.isArray(body?.lines) || !body.lines.length) {
    throw createError({ statusCode: 400, statusMessage: 'at least 1 line is required' })
  }
  return coreClient(event).createBom(id, {
    product_id: body.product_id.trim(),
    name: body.name.trim(),
    lines: body.lines
  })
})
