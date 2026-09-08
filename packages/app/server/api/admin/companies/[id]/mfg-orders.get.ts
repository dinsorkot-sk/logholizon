import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listMfgOrders(String(getRouterParam(event, 'id') || '')))
