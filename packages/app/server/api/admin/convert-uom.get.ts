import { coreClient } from '../../core/client'

export default defineEventHandler((event) => {
  const query = getQuery(event)
  const companyId = typeof query.company_id === 'string' ? query.company_id : ''
  const fromUomId = typeof query.from_uom_id === 'string' ? query.from_uom_id : ''
  const toUomId = typeof query.to_uom_id === 'string' ? query.to_uom_id : ''
  const qty = query.qty ? Number(query.qty) : NaN
  if (!companyId.trim() || !fromUomId.trim() || !toUomId.trim() || !Number.isFinite(qty)) {
    throw createError({ statusCode: 400, statusMessage: 'company_id, qty, from_uom_id, and to_uom_id are required' })
  }
  return coreClient(event).convertUom({ company_id: companyId, qty, from_uom_id: fromUomId, to_uom_id: toUomId })
})
