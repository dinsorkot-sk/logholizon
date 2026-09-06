import { coreClient } from '../../../core/client'

type UpdateRuleBody = { trigger?: string; target_url?: string; active?: boolean }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateRuleBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  return coreClient(event).updateNotificationRule(id, {
    trigger: body?.trigger,
    target_url: body?.target_url,
    active: body?.active
  })
})
