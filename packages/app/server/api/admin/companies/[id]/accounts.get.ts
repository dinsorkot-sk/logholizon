import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listGlAccounts(String(getRouterParam(event, 'id') || '')))
