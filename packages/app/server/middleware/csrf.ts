/**
 * CSRF Protection Middleware
 *
 * Enforces Origin/Referer validation for state-changing requests (POST/PUT/PATCH/DELETE)
 * on `/api/*` routes, protecting against cross-site request forgery attacks.
 *
 * Policy:
 * - GET/HEAD/OPTIONS: always allowed (safe methods, no state change)
 * - Public auth endpoints (/api/auth/login, /api/auth/register): allowed
 *   (these establish the session and are always safe to call)
 * - Mutating requests (POST/PUT/PATCH/DELETE): require matching Origin header
 * - Missing Origin header: treated as same-origin (browser-initiated, safe)
 * - Mismatched Origin: return 403 Forbidden
 */

import { defineEventHandler, getHeader } from 'h3'
import { TLSSocket } from 'node:tls'

export default defineEventHandler((event) => {
  const method = event.node.req.method
  const path = event.node.req.url || ''

  // Skip CSRF check for safe methods
  if (!method || ['GET', 'HEAD', 'OPTIONS'].includes(method)) {
    return
  }

  // Skip CSRF check for public auth endpoints
  // (they're called before session is fully established)
  if (path.startsWith('/api/auth/login') || path.startsWith('/api/auth/register')) {
    return
  }

  // Skip CSRF check for non-API routes
  if (!path.startsWith('/api/')) {
    return
  }

  // For mutating requests on /api/*, validate Origin header
  const origin = getHeader(event, 'origin')
  const referer = getHeader(event, 'referer')

  // If neither Origin nor Referer is present, treat as same-origin
  // (browser security prevents both from being stripped in cross-origin context)
  if (!origin && !referer) {
    return
  }

  // Build expected origin(s)
  const host = getHeader(event, 'host')
  const proto =
    getHeader(event, 'x-forwarded-proto') || // reverse proxy (production)
    (event.node.req.socket instanceof TLSSocket ? 'https' : 'http')

  const expectedOrigin = `${proto}://${host}`

  // Check Origin header first (preferred, sent by browsers for fetch/XHR)
  if (origin) {
    if (origin !== expectedOrigin) {
      throw createError({
        statusCode: 403,
        statusMessage: 'CSRF validation failed: origin mismatch',
      })
    }
    return
  }

  // Fallback: check Referer header (if Origin is not present)
  if (referer) {
    const refererUrl = new URL(referer, expectedOrigin)
    const refererOrigin = refererUrl.origin

    if (refererOrigin !== expectedOrigin) {
      throw createError({
        statusCode: 403,
        statusMessage: 'CSRF validation failed: referer mismatch',
      })
    }
    return
  }
})
