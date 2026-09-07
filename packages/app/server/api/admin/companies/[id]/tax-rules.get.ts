import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listTaxRules(String(getRouterParam(event, 'id') || '')))
