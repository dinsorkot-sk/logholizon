import { coreClient } from '../../../../core/client'

export default defineEventHandler((event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const query = getQuery(event)
  return coreClient(event).listStockLedger(id, {
    product_id: typeof query.product_id === 'string' ? query.product_id : undefined,
    warehouse_id: typeof query.warehouse_id === 'string' ? query.warehouse_id : undefined,
    limit: query.limit ? Number(query.limit) : undefined
  })
})
