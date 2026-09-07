import { coreClient } from '../../../../core/client'

export default defineEventHandler((event) => {
  const id = String(getRouterParam(event, 'id') || '')
  const query = getQuery(event)
  return coreClient(event).listTradeDocs(id, typeof query.doc_type === 'string' ? query.doc_type : undefined)
})
