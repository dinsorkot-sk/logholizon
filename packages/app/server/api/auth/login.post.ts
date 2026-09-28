import { TLSSocket } from 'node:tls'
import { coreClient } from '../../core/client'

type LoginBody = { username: string; password: string }

/**
 * Whether the current request actually arrived over HTTPS.
 *
 * IMPORTANT: this must be determined at request time, not via
 * `process.env.NODE_ENV`. Vite/Rollup inlines `process.env.NODE_ENV` as a
 * literal string at build time, so `nuxt build` (which always builds in
 * production mode) bakes in `secure: true` regardless of how the resulting
 * server is actually served. That breaks any deployment/test run that serves
 * the built output over plain HTTP (e.g. `nuxt preview` in CI/e2e), because
 * browsers silently refuse to store or send cookies marked `Secure` over a
 * non-HTTPS connection - every subsequent request then looks unauthenticated.
 */
function isSecureRequest(event: Parameters<typeof getHeader>[0]) {
  const proto = getHeader(event, 'x-forwarded-proto') // reverse proxy (production)
  if (proto) return proto === 'https'
  return event.node.req.socket instanceof TLSSocket
}

export default defineEventHandler(async (event) => {
  const body = await readBody<LoginBody>(event)
  if (!body?.username?.trim() || !body?.password) {
    throw createError({ statusCode: 400, statusMessage: 'username and password are required' })
  }
  const session = await coreClient().login(body.username, body.password)
  setCookie(event, 'lh_session', session.token, {
    httpOnly: true,
    secure: isSecureRequest(event),
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7
  })
  return { user: session.user }
})
