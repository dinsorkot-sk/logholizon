import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listBoms(String(getRouterParam(event, 'id') || '')))
