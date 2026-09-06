import { coreClient } from '../../core/client'

type CreateUserBody = { username: string; password: string; role?: string }

export default defineEventHandler(async (event) => {
  const body = await readBody<CreateUserBody>(event)
  if (!body?.username?.trim() || !body?.password) {
    throw createError({ statusCode: 400, statusMessage: 'username and password are required' })
  }
  return coreClient(event).createUser({
    username: body.username,
    password: body.password,
    role: body.role || 'user'
  })
})