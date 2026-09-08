import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listPosOrders(String(getRouterParam(event, 'id') || '')))
