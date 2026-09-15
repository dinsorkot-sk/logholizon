// Desktop fetch layer: routes `/api/*` calls directly to the in-process
// Rust core (`/v1/*`) when the app runs inside Tauri. On web this module is
// inert (returns null) and the Nitro gateway handles `/api/*` as before.
//
// Wiring: the `desktop` Nuxt module replaces `#build/fetch.mjs` at build
// time so the global `$fetch` (and therefore every `useFetch`/`$fetch`
// call site) goes through `desktopFetch` first. Non-API URLs and web mode
// fall through to the original ofetch instance untouched.

import { ofetch } from 'ofetch'
import { resolveDesktopRoute, validateDesktopBody } from './desktop-routes'

export type DesktopContext = {
  /** Core base URL, e.g. `http://127.0.0.1:54321` (ephemeral sidecar port). */
  baseUrl: string
  /** Stored Bearer token (Tauri secure store), or null when logged out. */
  token: string | null
}

type FetchOptions = Record<string, any>

function isDesktop(): boolean {
  if (typeof window === 'undefined') return false
  const w = window as any
  return !!(
    w.__TAURI__ ||
    w.__TAURI_INTERNALS__ ||
    (window as any).__LOGHOLIZON_DESKTOP__ ||
    // Tauri devUrl mode: plain browser pointed at the desktop Nuxt dev
    // server (LOGHOLIZON_DESKTOP=1). The sidecar port comes from
    // NUXT_PUBLIC_DESKTOP_CORE_URL (see nuxt.config.ts).
    (window as any).__NUXT__?.config?.public?.desktop
  )
}

function desktopContext(): DesktopContext | null {
  if (!isDesktop()) return null
  const w = window as any
  const publicConfig = (w.__NUXT__?.config?.public || {}) as Record<string, unknown>
  const baseUrl =
    w.__LOGHOLIZON_CORE_URL__ ||
    (typeof publicConfig.desktopCoreUrl === 'string' && publicConfig.desktopCoreUrl) ||
    'http://127.0.0.1:8787'
  const token = w.__LOGHOLIZON_TOKEN__ || localStorage.getItem('lh_desktop_token')
  return { baseUrl: String(baseUrl).replace(/\/$/, ''), token }
}

function toCoreError(status: number, body: any): Error & { data?: any; statusCode?: number; statusMessage?: string } {
  const message = body?.message || (status === 401 ? 'not authenticated' : `request failed (${status})`)
  const error = new Error(message) as Error & { data?: any; statusCode?: number; statusMessage?: string }
  error.data = { code: body?.code || 'core_error', message }
  error.statusCode = status
  error.statusMessage = message
  return error
}

async function coreFetch(ctx: DesktopContext, corePath: string, options: FetchOptions): Promise<any> {
  const method = (options?.method || 'GET').toUpperCase()
  const headers: Record<string, string> = { ...(options?.headers || {}) }
  if (ctx.token && !headers.authorization && !headers.Authorization) {
    headers.authorization = `Bearer ${ctx.token}`
  }
  let body: BodyInit | undefined
  const contentType = headers['content-type'] || headers['Content-Type'] || ''
  if (options?.body !== undefined) {
    if (options.body instanceof Uint8Array || options.body instanceof ArrayBuffer) {
      body = options.body as BodyInit
      if (!contentType) headers['content-type'] = 'application/octet-stream'
    } else if (typeof options.body === 'string') {
      body = options.body
      if (!contentType) headers['content-type'] = 'text/plain;charset=utf-8'
    } else {
      body = JSON.stringify(options.body)
      if (!contentType) headers['content-type'] = 'application/json'
    }
  } else if (method !== 'GET' && method !== 'HEAD' && !contentType) {
    headers['content-type'] = 'application/json'
  }

  const query = options?.query || options?.params
  const suffix = query ? `?${new URLSearchParams(query as Record<string, string>).toString()}` : ''
  const response = await fetch(`${ctx.baseUrl}${corePath}${suffix}`, { method, headers, body })
  const status = response.status

  if (options?.responseType === 'arrayBuffer') {
    if (!response.ok) throw toCoreError(status, await response.json().catch(() => null))
    return response.arrayBuffer()
  }
  // Binary downloads (workbook/attachment/backup): gateway returns Response.
  const responseContentType = response.headers.get('content-type') || ''
  if (!responseContentType.includes('application/json')) {
    if (!response.ok) throw toCoreError(status, null)
    return response.arrayBuffer()
  }
  const data = await response.json().catch(() => null)
  if (!response.ok) throw toCoreError(status, data)
  return data
}

/** Split `/api/path?query` into path + query string. */
function splitUrl(request: string): { path: string; query: string } {
  const index = request.indexOf('?')
  if (index === -1) return { path: request, query: '' }
  return { path: request.slice(0, index), query: request.slice(index) }
}

/**
 * Desktop-aware `$fetch` replacement. Returns the core response for
 * `/api/*` calls when running inside Tauri; otherwise delegates to the
 * original ofetch instance. Solution catalog routes are served from the
 * bundled JSON (same files the gateway imports).
 */
export function createDesktopFetch(original: typeof ofetch, solutions: { catalog: () => unknown[]; package: (name: string) => unknown | null }) {
  return async function desktopFetch(request: string, options: FetchOptions = {}): Promise<any> {
    if (typeof request !== 'string' || !request.startsWith('/api/')) {
      return original(request, options)
    }
    const ctx = desktopContext()
    if (!ctx) return original(request, options)

    const { path, query } = splitUrl(request)

    // Solution Library catalog is bundled JSON (no core round-trip).
    if (path === '/api/solutions' || path === '/api/solutions/') {
      return solutions.catalog()
    }
    const solutionMatch = /^\/api\/solutions\/([^/]+)$/.exec(path)
    if (solutionMatch) {
      const pkg = solutions.package(solutionMatch[1] || '')
      if (!pkg) throw toCoreError(404, { code: 'not_found', message: `unknown solution: ${solutionMatch[1]}` })
      return pkg
    }

    const route = resolveDesktopRoute(path)
    if (!route) {
      throw toCoreError(404, { code: 'not_found', message: `unknown desktop route: ${path}` })
    }

    // Mirror gateway validation so pages see the same 400 messages as web.
    if (options?.body !== undefined) {
      const message = validateDesktopBody(path, options.body)
      if (message) throw toCoreError(400, { code: 'bad_request', message })
    }

    // Auth endpoints: login/register return the session (caller persists
    // the token); me/logout need the stored token like the gateway cookie.
    const method = (options?.method || route.method || 'GET').toUpperCase()
    return coreFetch(ctx, route.core + query, { ...options, method })
  }
}

/** Persist the desktop token (localStorage; Tauri secure store when available). */
export function readDesktopToken(): string | null {
  if (typeof window === 'undefined') return null
  const w = window as any
  return w.__LOGHOLIZON_TOKEN__ || localStorage.getItem('lh_desktop_token')
}

export function writeDesktopToken(token: string | null) {
  if (typeof window === 'undefined') return
  const w = window as any
  if (token) {
    w.__LOGHOLIZON_TOKEN__ = token
    localStorage.setItem('lh_desktop_token', token)
  } else {
    delete w.__LOGHOLIZON_TOKEN__
    localStorage.removeItem('lh_desktop_token')
  }
}

export function isDesktopMode(): boolean {
  return isDesktop()
}
