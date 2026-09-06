import { coreClient } from '../../../../core/client'

type ResetPasswordBody = { password: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<ResetPasswordBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.password) throw createError({ statusCode: 400, statusMessage: 'password is required' })
  await coreClient(event).resetUserPassword(id, body.password)
  return { success: true }
})