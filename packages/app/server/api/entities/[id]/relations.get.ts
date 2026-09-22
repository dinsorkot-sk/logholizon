import { getRouterParam } from 'h3'
import { coreClient } from '../../../core/client'

export default defineEventHandler(event => {
  const id = getRouterParam(event, 'id') || ''
  return coreClient(event).listRelations(id)
})
