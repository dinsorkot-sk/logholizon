import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).activateBom(String(getRouterParam(event, 'id') || '')))
