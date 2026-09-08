import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listLeaveRequests(String(getRouterParam(event, 'id') || '')))
