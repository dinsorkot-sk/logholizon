import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody(event)
  const trigger = String(body?.trigger || '').trim()
  const targetUrl = String(body?.target_url || '').trim()
  if (!trigger || !targetUrl) {
    throw createError({ statusCode: 400, statusMessage: 'trigger and target_url are required' })
  }
  return coreClient(event).createAutomation(id, {
    trigger,
    action: body?.action || 'webhook',
    target_url: targetUrl,
    active: body?.active !== false
  })
})
