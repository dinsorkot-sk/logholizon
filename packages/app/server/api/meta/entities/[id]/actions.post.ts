import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody(event)
  const name = String(body?.name || '').trim()
  const label = String(body?.label || '').trim()
  const kind = String(body?.kind || '').trim()
  if (!name || !label || !kind) {
    throw createError({ statusCode: 400, statusMessage: 'name, label and kind are required' })
  }
  return coreClient(event).createModuleAction(id, { name, label, kind, config: body?.config || {} })
})
