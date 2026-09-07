import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listUoms(String(getRouterParam(event, 'id') || '')))
