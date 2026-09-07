import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listStockBalances(String(getRouterParam(event, 'id') || '')))
