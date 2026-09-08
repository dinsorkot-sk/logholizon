import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).voidPosOrder(String(getRouterParam(event, 'id') || '')))
