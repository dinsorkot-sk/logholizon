import { describe, expect, it } from 'vitest'
import { resolveDesktopRoute } from '../app/utils/desktop-routes'

describe('resolveDesktopRoute', () => {
  it('maps auth endpoints to /v1/auth', () => {
    expect(resolveDesktopRoute('/api/auth/login')).toEqual({ core: '/v1/auth/login', method: 'POST' })
    expect(resolveDesktopRoute('/api/auth/me')).toEqual({ core: '/v1/auth/me', method: undefined })
    expect(resolveDesktopRoute('/api/auth/status')).toEqual({ core: '/v1/auth/status', method: undefined })
  })

  it('maps entity and document routes with params', () => {
    expect(resolveDesktopRoute('/api/entities')).toEqual({ core: '/v1/entities', method: undefined })
    expect(resolveDesktopRoute('/api/entities/wo')).toEqual({ core: '/v1/entities/wo', method: undefined })
    expect(resolveDesktopRoute('/api/documents/abc/transition')).toEqual({
      core: '/v1/documents/abc/transition',
      method: 'POST'
    })
    expect(resolveDesktopRoute('/api/documents/abc/audit')).toEqual({
      core: '/v1/documents/abc/audit',
      method: undefined
    })
  })

  it('maps meta, module, and admin routes', () => {
    expect(resolveDesktopRoute('/api/meta/entities')).toEqual({ core: '/v1/meta/entities', method: undefined })
    expect(resolveDesktopRoute('/api/meta/entities/wo/views')).toEqual({
      core: '/v1/meta/entities/wo/views',
      method: undefined
    })
    expect(resolveDesktopRoute('/api/modules')).toEqual({ core: '/v1/modules', method: undefined })
    expect(resolveDesktopRoute('/api/modules/m1/enable')).toEqual({
      core: '/v1/modules/m1/enable',
      method: 'POST'
    })
    expect(resolveDesktopRoute('/api/modules/packages/install')).toEqual({
      core: '/v1/modules/packages/install',
      method: 'POST'
    })
    expect(resolveDesktopRoute('/api/admin/observability/metrics')).toEqual({
      core: '/v1/admin/observability/metrics',
      method: undefined
    })
  })

  it('maps module dynamic documents and relation links', () => {
    expect(resolveDesktopRoute('/api/modules/sales/entities/order')).toEqual({
      core: '/v1/modules/sales/entities/order',
      method: undefined
    })
    expect(resolveDesktopRoute('/api/modules/sales/entities/order/o1')).toEqual({
      core: '/v1/modules/sales/entities/order/o1',
      method: undefined
    })
    expect(resolveDesktopRoute('/api/meta/relations/r1/links/d1')).toEqual({
      core: '/v1/meta/relations/r1/links/d1',
      method: undefined
    })
  })

  it('returns null for unknown routes', () => {
    expect(resolveDesktopRoute('/api/nope')).toBeNull()
    expect(resolveDesktopRoute('/other/path')).toBeNull()
  })
})
