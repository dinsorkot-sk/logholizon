export default defineEventHandler((event) => {
  const headers = event.node.res
  headers.setHeader('X-Content-Type-Options', 'nosniff')
  headers.setHeader('X-Frame-Options', 'DENY')
  headers.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin')
  headers.setHeader('Permissions-Policy', 'camera=(), microphone=(), geolocation=()')

  if (process.env.NODE_ENV === 'production') {
    headers.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains')
  }

  const method = event.node.req.method || 'GET'
  const origin = getHeader(event, 'origin')
  if (origin && ['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
    if (origin !== getRequestURL(event).origin) {
      throw createError({ statusCode: 403, statusMessage: 'cross-origin mutation denied' })
    }
  }
})
