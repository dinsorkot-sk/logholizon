import { coreClient } from '../../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody<{ event_type?: unknown; document_id?: unknown; payload?: unknown }>(event)
  const eventType = String(body?.event_type || '').trim()
  if (!eventType) {
    throw createError({ statusCode: 400, statusMessage: 'event_type is required' })
  }
  return coreClient(event).deliverWebhook(id, {
    event_type: eventType,
    document_id: typeof body?.document_id === 'string' ? body.document_id : undefined,
    payload: (body?.payload || {}) as Record<string, unknown>
  })
})
