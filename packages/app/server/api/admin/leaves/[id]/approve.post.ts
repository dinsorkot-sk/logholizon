import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).approveLeave(String(getRouterParam(event, 'id') || '')))
