import { coreClient } from '../../core/client'

type RegisterBody = { username: string; password: string }

export default defineEventHandler(async (event) => {
  const body = await readBody<RegisterBody>(event)
  if (!body?.username?.trim() || !body?.password) {
    throw createError({ statusCode: 400, statusMessage: 'username and password are required' })
  }
  return coreClient().register(body.username, body.password)
})