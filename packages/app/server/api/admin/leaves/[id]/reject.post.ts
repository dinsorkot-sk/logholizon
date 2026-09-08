import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).rejectLeave(String(getRouterParam(event, 'id') || '')))
