import { coreClient } from '../../core/client'

type LoginBody = { username: string; password: string }

export default defineEventHandler(async (event) => {
  const body = await readBody<LoginBody>(event)
  if (!body?.username?.trim() || !body?.password) {
    throw createError({ statusCode: 400, statusMessage: 'username and password are required' })
  }
  const session = await coreClient().login(body.username, body.password)
  setCookie(event, 'lh_session', session.token, {
    httpOnly: true,
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7
  })
  return { user: session.user }
})