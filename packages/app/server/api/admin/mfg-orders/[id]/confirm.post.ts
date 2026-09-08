import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).confirmMfgOrder(String(getRouterParam(event, 'id') || '')))
