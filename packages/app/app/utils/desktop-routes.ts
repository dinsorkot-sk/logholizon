// Desktop API routing table: Nitro gateway path -> Rust core /v1 path.
//
// The web app calls relative `/api/*` routes served by the Nitro gateway,
// which validates input then forwards to the Rust core (`server/core/client.ts`).
// The desktop static SPA has no Nitro server, so the desktop fetch layer
// rewrites each `/api/*` call to the equivalent core `/v1/*` endpoint
// directly, preserving the gateway's validation contracts (see VALIDATION).
//
// Routes NOT listed here are handled locally by the desktop layer:
// - `/api/solutions` + `/api/solutions/:name` are bundled JSON (imported
//   from `packages/erp/*.module.json` at build time, same as the gateway).
// - `/api/auth/login` + `/api/auth/register` + `/api/auth/status` map
//   1:1 to `/v1/auth/*` (no cookie handling; the token is returned to the
//   caller and persisted in the desktop secure store).
// - `/api/auth/me` + `/api/auth/logout` require the stored Bearer token.

export type DesktopRoute = {
  /** Core path, possibly with `:param` placeholders filled from the URL. */
  core: string
  /** HTTP method override (gateway method differs from core method). */
  method?: string
  /** Query keys dropped before forwarding (gateway-only params). */
  dropQuery?: string[]
}

// Static GET routes (no params).
const STATIC_GET: Record<string, string> = {
  '/api/entities': '/v1/entities',
  '/api/meta/entities': '/v1/meta/entities',
  '/api/meta/dashboards': '/v1/meta/dashboards',
  '/api/meta/webhooks': '/v1/meta/webhooks',
  '/api/meta/notification-templates': '/v1/meta/notification-templates',
  '/api/modules': '/v1/modules',
  '/api/audit': '/v1/audit',
  '/api/documents': '/v1/documents',
  '/api/notifications': '/v1/notifications',
  '/api/reports/aggregate': '/v1/reports/aggregate',
  '/api/entities/export': '/v1/entities/export',
  '/api/admin/users': '/v1/admin/users',
  '/api/admin/roles': '/v1/admin/roles',
  '/api/admin/status': '/v1/admin/status',
  '/api/admin/backups': '/v1/admin/backups',
  '/api/admin/notification-deliveries': '/v1/admin/notification-deliveries',
  '/api/admin/observability/logs': '/v1/admin/observability/logs',
  '/api/admin/observability/metrics': '/v1/admin/observability/metrics',
  '/api/auth/status': '/v1/auth/status'
}

// Dynamic routes with `:param` segments, matched in order.
const DYNAMIC: { pattern: RegExp; keys: string[]; core: string; method?: string }[] = [
  // Auth (token-based; no cookies on desktop).
  { pattern: /^\/api\/auth\/login$/, keys: [], core: '/v1/auth/login', method: 'POST' },
  { pattern: /^\/api\/auth\/register$/, keys: [], core: '/v1/auth/register', method: 'POST' },
  { pattern: /^\/api\/auth\/me$/, keys: [], core: '/v1/auth/me' },
  { pattern: /^\/api\/auth\/logout$/, keys: [], core: '/v1/auth/logout', method: 'POST' },
  // Documents.
  { pattern: /^\/api\/documents\/([^/]+)\/workflow-history$/, keys: ['id'], core: '/v1/documents/:id/workflow-history' },
  { pattern: /^\/api\/documents\/([^/]+)\/transition$/, keys: ['id'], core: '/v1/documents/:id/transition', method: 'POST' },
  { pattern: /^\/api\/documents\/([^/]+)\/followers$/, keys: ['id'], core: '/v1/documents/:id/followers' },
  { pattern: /^\/api\/documents\/([^/]+)\/comments$/, keys: ['id'], core: '/v1/documents/:id/comments' },
  { pattern: /^\/api\/documents\/([^/]+)\/activities$/, keys: ['id'], core: '/v1/documents/:id/activities' },
  { pattern: /^\/api\/documents\/([^/]+)\/attachments$/, keys: ['id'], core: '/v1/documents/:id/attachments' },
  { pattern: /^\/api\/documents\/([^/]+)\/audit$/, keys: ['id'], core: '/v1/documents/:id/audit' },
  { pattern: /^\/api\/documents\/([^/]+)$/, keys: ['id'], core: '/v1/documents/:id' },
  { pattern: /^\/api\/documents$/, keys: [], core: '/v1/documents' },
  { pattern: /^\/api\/activities\/([^/]+)\/toggle$/, keys: ['id'], core: '/v1/activities/:id/toggle', method: 'POST' },
  { pattern: /^\/api\/attachments\/([^/]+)\/download$/, keys: ['id'], core: '/v1/attachments/:id' },
  { pattern: /^\/api\/attachments\/([^/]+)$/, keys: ['id'], core: '/v1/attachments/:id' },
  // User-facing entity routes.
  { pattern: /^\/api\/entities\/([^/]+)\/relations\/([^/]+)$/, keys: ['id', 'relation_id'], core: '/v1/entities/:id/relations/:relation_id' },
  { pattern: /^\/api\/entities\/([^/]+)\/actions\/([^/]+)$/, keys: ['id', 'action_id'], core: '/v1/entities/:id/actions/:action_id', method: 'POST' },
  { pattern: /^\/api\/entities\/([^/]+)\/export$/, keys: ['id'], core: '/v1/entities/:id/export' },
  { pattern: /^\/api\/entities\/([^/]+)\/form-layout$/, keys: ['id'], core: '/v1/entities/:id/form-layout' },
  { pattern: /^\/api\/entities\/([^/]+)\/import-confirm$/, keys: ['id'], core: '/v1/entities/:id/import/confirm', method: 'POST' },
  { pattern: /^\/api\/entities\/([^/]+)\/import-preview$/, keys: ['id'], core: '/v1/entities/:id/import/preview', method: 'POST' },
  { pattern: /^\/api\/entities\/([^/]+)\/options$/, keys: ['id'], core: '/v1/entities/:id/options' },
  { pattern: /^\/api\/entities\/([^/]+)\/reports$/, keys: ['id'], core: '/v1/entities/:id/reports' },
  { pattern: /^\/api\/entities\/([^/]+)\/views$/, keys: ['id'], core: '/v1/entities/:id/views' },
  { pattern: /^\/api\/entities\/([^/]+)\/workflow$/, keys: ['id'], core: '/v1/entities/:id/workflow' },
  { pattern: /^\/api\/entities\/([^/]+)$/, keys: ['id'], core: '/v1/entities/:id' },
  { pattern: /^\/api\/entities\/import-confirm$/, keys: [], core: '/v1/entities/import/confirm', method: 'POST' },
  { pattern: /^\/api\/entities\/import-preview$/, keys: [], core: '/v1/entities/import/preview', method: 'POST' },
  { pattern: /^\/api\/views\/([^/]+)$/, keys: ['id'], core: '/v1/views/:id' },
  { pattern: /^\/api\/dashboards\/([^/]+)$/, keys: ['id'], core: '/v1/dashboards/:id' },
  { pattern: /^\/api\/reports\/aggregate$/, keys: [], core: '/v1/reports/aggregate' },
  { pattern: /^\/api\/reports\/([^/]+)\/run$/, keys: ['id'], core: '/v1/reports/:id/run', method: 'POST' },
  { pattern: /^\/api\/reports\/([^/]+)$/, keys: ['id'], core: '/v1/reports/:id' },
  { pattern: /^\/api\/dashboard\/counts$/, keys: [], core: '/v1/dashboard/counts' },
  { pattern: /^\/api\/dashboard\/pm$/, keys: [], core: '/v1/dashboard/pm' },
  { pattern: /^\/api\/notifications\/([^/]+)\/read$/, keys: ['id'], core: '/v1/notifications/:id/read', method: 'POST' },
  // Meta (admin) routes.
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/workflow\/transitions$/, keys: ['id'], core: '/v1/meta/entities/:id/workflow/transitions', method: 'POST' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/workflow\/states$/, keys: ['id'], core: '/v1/meta/entities/:id/workflow/states', method: 'POST' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/workflow$/, keys: ['id'], core: '/v1/meta/entities/:id/workflow' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/permissions$/, keys: ['id'], core: '/v1/meta/entities/:id/permissions' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/field-permissions$/, keys: ['id'], core: '/v1/meta/entities/:id/field-permissions' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/views$/, keys: ['id'], core: '/v1/meta/entities/:id/views' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/form-layout$/, keys: ['id'], core: '/v1/meta/entities/:id/form-layout' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/notification-rules$/, keys: ['id'], core: '/v1/meta/entities/:id/notification-rules' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/actions$/, keys: ['id'], core: '/v1/meta/entities/:id/actions' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/automations$/, keys: ['id'], core: '/v1/meta/entities/:id/automations' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/events$/, keys: ['id'], core: '/v1/meta/entities/:id/events' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/export$/, keys: ['id'], core: '/v1/meta/entities/:id/export' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/import-confirm$/, keys: ['id'], core: '/v1/meta/entities/:id/import/confirm', method: 'POST' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/import-preview$/, keys: ['id'], core: '/v1/meta/entities/:id/import/preview', method: 'POST' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/fields$/, keys: ['id'], core: '/v1/meta/entities/:id/fields', method: 'POST' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/relations$/, keys: ['id'], core: '/v1/meta/entities/:id/relations' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)\/reports$/, keys: ['id'], core: '/v1/meta/entities/:id/reports' },
  { pattern: /^\/api\/meta\/entities\/([^/]+)$/, keys: ['id'], core: '/v1/meta/entities/:id' },
  { pattern: /^\/api\/meta\/entities$/, keys: [], core: '/v1/meta/entities' },
  { pattern: /^\/api\/meta\/actions\/([^/]+)$/, keys: ['id'], core: '/v1/meta/actions/:id' },
  { pattern: /^\/api\/meta\/automations\/([^/]+)\/executions$/, keys: ['id'], core: '/v1/meta/automations/:id/executions' },
  { pattern: /^\/api\/meta\/automations\/([^/]+)$/, keys: ['id'], core: '/v1/meta/automations/:id' },
  { pattern: /^\/api\/meta\/views\/([^/]+)$/, keys: ['id'], core: '/v1/meta/views/:id' },
  { pattern: /^\/api\/meta\/fields\/([^/]+)\/options$/, keys: ['id'], core: '/v1/meta/fields/:id/options', method: 'POST' },
  { pattern: /^\/api\/meta\/fields\/([^/]+)$/, keys: ['id'], core: '/v1/meta/fields/:id' },
  { pattern: /^\/api\/meta\/options\/([^/]+)$/, keys: ['id'], core: '/v1/meta/options/:id' },
  { pattern: /^\/api\/meta\/notification-rules\/([^/]+)$/, keys: ['id'], core: '/v1/meta/notification-rules/:id' },
  { pattern: /^\/api\/meta\/notification-templates\/([^/]+)$/, keys: ['id'], core: '/v1/meta/notification-templates/:id' },
  { pattern: /^\/api\/meta\/webhooks\/([^/]+)\/deliver$/, keys: ['id'], core: '/v1/meta/webhooks/:id/deliver', method: 'POST' },
  { pattern: /^\/api\/meta\/relations\/([^/]+)\/links\/([^/]+)$/, keys: ['id', 'source_doc_id'], core: '/v1/meta/relations/:id/links/:source_doc_id' },
  { pattern: /^\/api\/meta\/relations\/([^/]+)$/, keys: ['id'], core: '/v1/meta/relations/:id' },
  { pattern: /^\/api\/meta\/reports\/([^/]+)$/, keys: ['id'], core: '/v1/meta/reports/:id' },
  { pattern: /^\/api\/meta\/dashboards\/([^/]+)$/, keys: ['id'], core: '/v1/meta/dashboards/:id' },
  { pattern: /^\/api\/meta\/workflow\/states\/([^/]+)$/, keys: ['id'], core: '/v1/meta/workflow/states/:id' },
  { pattern: /^\/api\/meta\/workflow\/transitions\/([^/]+)$/, keys: ['id'], core: '/v1/meta/workflow/transitions/:id' },
  // Modules.
  { pattern: /^\/api\/modules\/packages\/preview$/, keys: [], core: '/v1/modules/packages/preview', method: 'POST' },
  { pattern: /^\/api\/modules\/packages\/install$/, keys: [], core: '/v1/modules/packages/install', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/package\/uninstall$/, keys: ['id'], core: '/v1/modules/:id/package/uninstall', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/package$/, keys: ['id'], core: '/v1/modules/:id/package' },
  { pattern: /^\/api\/modules\/([^/]+)\/manifest$/, keys: ['id'], core: '/v1/modules/:id/manifest' },
  { pattern: /^\/api\/modules\/([^/]+)\/versions$/, keys: ['id'], core: '/v1/modules/:id/versions' },
  { pattern: /^\/api\/modules\/([^/]+)\/rollback$/, keys: ['id'], core: '/v1/modules/:id/rollback', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/review$/, keys: ['id'], core: '/v1/modules/:id/review', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/publish$/, keys: ['id'], core: '/v1/modules/:id/publish', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/enable$/, keys: ['id'], core: '/v1/modules/:id/enable', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/disable$/, keys: ['id'], core: '/v1/modules/:id/disable', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/archive$/, keys: ['id'], core: '/v1/modules/:id/archive', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/restore$/, keys: ['id'], core: '/v1/modules/:id/restore', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/entities\/([^/]+)\/bulk-delete$/, keys: ['module', 'entity'], core: '/v1/modules/:module/entities/:entity/bulk-delete', method: 'POST' },
  { pattern: /^\/api\/modules\/([^/]+)\/entities\/([^/]+)\/([^/]+)$/, keys: ['module', 'entity', 'id'], core: '/v1/modules/:module/entities/:entity/:id' },
  { pattern: /^\/api\/modules\/([^/]+)\/entities\/([^/]+)$/, keys: ['module', 'entity'], core: '/v1/modules/:module/entities/:entity' },
  { pattern: /^\/api\/modules\/([^/]+)$/, keys: ['id'], core: '/v1/modules/:id' },
  // Admin users/roles.
  { pattern: /^\/api\/admin\/users\/([^/]+)\/reset-password$/, keys: ['id'], core: '/v1/admin/users/:id/reset-password', method: 'POST' },
  { pattern: /^\/api\/admin\/users\/([^/]+)$/, keys: ['id'], core: '/v1/admin/users/:id' },
  { pattern: /^\/api\/admin\/roles\/([^/]+)$/, keys: ['id'], core: '/v1/admin/roles/:id' },
  { pattern: /^\/api\/admin\/backups\/([^/]+)$/, keys: ['name'], core: '/v1/admin/backups/:name' },
  { pattern: /^\/api\/admin\/backup$/, keys: [], core: '/v1/admin/backup', method: 'POST' },
  { pattern: /^\/api\/admin\/restore$/, keys: [], core: '/v1/admin/restore', method: 'POST' },
  { pattern: /^\/api\/admin\/restart$/, keys: [], core: '/v1/admin/restart', method: 'POST' }
]

export function resolveDesktopRoute(path: string): DesktopRoute | null {
  const staticCore = STATIC_GET[path]
  if (staticCore) return { core: staticCore }
  for (const route of DYNAMIC) {
    const match = route.pattern.exec(path)
    if (!match) continue
    let core = route.core
    for (let i = 0; i < route.keys.length; i++) {
      core = core.replace(`:${route.keys[i]}`, encodeURIComponent(match[i + 1] || ''))
    }
    return { core, method: route.method }
  }
  return null
}

// Gateway input validation mirrored on desktop so invalid input fails fast
// with the same 400 status codes (core would reject most anyway, but the
// gateway checks run first on web and pages rely on their messages).
const VALIDATORS: { pattern: RegExp; validate: (body: any) => string | null }[] = [
  {
    pattern: /^\/api\/auth\/(login|register)$/,
    validate: (body) => (!body?.username?.trim() || !body?.password ? 'username and password are required' : null)
  },
  {
    pattern: /^\/api\/documents$/,
    validate: (body) => {
      if (!body || typeof body !== 'object') return null // GET list has no body
      if (body.id !== undefined && !String(body.id).trim()) return 'id and entity_id are required'
      return null
    }
  }
]

export function validateDesktopBody(path: string, body: unknown): string | null {
  for (const v of VALIDATORS) {
    if (v.pattern.test(path)) {
      const message = v.validate(body)
      if (message) return message
    }
  }
  return null
}
