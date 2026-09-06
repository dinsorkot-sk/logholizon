import { coreClient } from '../../../../core/client'

type CreateRuleBody = { trigger?: string; target_url: string; active?: boolean }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<CreateRuleBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.target_url?.trim()) throw createError({ statusCode: 400, statusMessage: 'target_url is required' })
  return coreClient(event).createNotificationRule(id, {
    trigger: body.trigger,
    target_url: body.target_url,
    active: body.active
  })
})
