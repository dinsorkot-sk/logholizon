export default defineEventHandler((event) => {
  const headers = event.node.res
  headers.setHeader('X-Content-Type-Options', 'nosniff')
  headers.setHeader('X-Frame-Options', 'DENY')
  headers.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin')
  headers.setHeader('Permissions-Policy', 'camera=(), microphone=(), geolocation=()')

  if (process.env.NODE_ENV === 'production') {
    headers.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains')
  }
})
