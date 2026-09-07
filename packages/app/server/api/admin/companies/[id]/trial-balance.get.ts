import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).getTrialBalance(String(getRouterParam(event, 'id') || '')))
