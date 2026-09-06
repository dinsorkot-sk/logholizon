export type CoreEntity = { id: string; name: string; label: string; module?: string | null }

export type CoreEntityOption = { id: string; label: string }

export type CoreUser = { id: string; username: string; role: string }
export type CoreSession = { token: string; user: CoreUser }
export type CoreUserRow = { id: string; username: string; role: string; created_at: string }

export type CoreFieldOption = { id: string; value: string; label: string }

export type CoreField = {
  id: string
  name: string
  type: string
  required: boolean
  is_status: boolean
  position: number
  ref_entity?: string | null
  computed_expr?: string | null
  options: CoreFieldOption[]
}

export type CoreEntityDetail = CoreEntity & { fields: CoreField[] }

export type CoreFieldWithPermission = CoreField & { can_view: boolean; can_edit: boolean }

export type CoreEntityWithPermission = CoreEntity & {
  fields: CoreFieldWithPermission[]
  permission: CoreEntityPermission
}

export type CoreFieldPermission = { field_id: string; role: string; can_view: boolean; can_edit: boolean }

export type CoreDocument = {
  id: string
  entity_id: string
  payload: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type CoreDocumentList = {
  items: CoreDocument[]
  total: number
}

export type CoreAuditEntry = {
  id: string
  entity_id: string
  doc_id: string
  action: string
  payload: Record<string, unknown>
  created_at: string
  actor?: string | null
}

export type CoreAuditList = {
  items: CoreAuditEntry[]
  total: number
}

export type CoreGlobalAuditEntry = {
  id: string
  entity_id: string
  entity_label: string
  doc_id: string
  action: string
  payload: Record<string, unknown>
  created_at: string
  actor?: string | null
}

export type CoreGlobalAuditList = {
  items: CoreGlobalAuditEntry[]
  total: number
}

export type CoreEntityPermission = { role: string; can_view: boolean; can_edit: boolean }
export type CoreEntityView = {
  id: string
  entity_id: string
  name: string
  config: Record<string, unknown>
  created_at: string
}

export type CoreFormLayout = {
  entity_id: string
  config: Record<string, unknown>
}

export type CoreNotificationRule = {
  id: string
  entity_id: string
  trigger: string
  target_url: string
  active: boolean
  created_at: string
}

export type CoreNotificationDelivery = {
  id: string
  rule_id: string
  document_id: string
  action: string
  payload: Record<string, unknown>
  status: string
  attempts: number
  last_error: string | null
  created_at: string
}

export type CoreNotificationDeliveryList = {
  items: CoreNotificationDelivery[]
  total: number
}

export type CoreReport = {
  id: string
  entity_id: string
  name: string
  config: Record<string, unknown>
  created_by: string | null
  created_at: string
}

export type CoreReportBucket = {
  status: string
  count: number
}

export type CreateDocumentInput = {
  id: string
  entity_id: string
  payload: Record<string, unknown>
}

export type CoreWorkflowState = { id: string; name: string; label: string; position: number }
export type CoreWorkflowTransition = { id: string; action: string; from_state: string; to_state: string }
export type CoreWorkflowDefinition = { states: CoreWorkflowState[]; transitions: CoreWorkflowTransition[] }
export type CoreStatusCount = { status: string; count: number }
export type CorePmSummary = { open: number; overdue: number; done_this_week: number; total: number }
export type CoreAdminStatus = { version: string; database_path: string; integrity: boolean; entities: number; documents: number }
export type CoreBackupInfo = { name: string; size: number; modified: number }
export type CoreMultiImportSheet = {
  entity_id: string
  rows: { id: string; payload: Record<string, unknown> }[]
  errors: string[]
}
export type CoreMultiImportPreview = { sheets: CoreMultiImportSheet[] }
export type CoreMultiImportSheetResult = { entity_id: string; created: number; updated: number }
export type CoreMultiImportResult = { sheets: CoreMultiImportSheetResult[] }

type CoreError = { code?: string; message?: string }

export function coreClient(event?: Parameters<typeof getCookie>[0]) {
  const config = useRuntimeConfig()
  const baseURL = config.coreUrl.replace(/\/$/, '')

  async function request<T>(path: string, options?: Parameters<typeof $fetch<T>>[1]): Promise<T> {
    try {
      // Forward the browser session cookie as a Bearer token so the Rust
      // core can authenticate gateway requests. Callers pass the Nitro
      // event explicitly (server utils are not auto-imported here).
      const token = event ? getCookie(event, 'lh_session') : undefined
      const headers = new Headers(options?.headers as HeadersInit | undefined)
      if (token && !headers.has('authorization')) {
        headers.set('authorization', `Bearer ${token}`)
      }
      return (await $fetch<T>(`${baseURL}${path}`, { ...options, headers })) as T
    } catch (error: any) {
      const body = error?.data as CoreError | undefined
      throw createError({
        statusCode: error?.statusCode || 502,
        statusMessage: body?.message || 'Rust core unavailable',
        data: { code: body?.code || 'core_unavailable' }
      })
    }
  }

  return {
    login: (username: string, password: string): Promise<CoreSession> =>
      request<CoreSession>('/v1/auth/login', { method: 'POST', body: { username, password } }),
    register: (username: string, password: string): Promise<CoreUser> =>
      request<CoreUser>('/v1/auth/register', { method: 'POST', body: { username, password } }),
    logout: (token: string): Promise<void> =>
      request<void>('/v1/auth/logout', { method: 'POST', headers: { authorization: `Bearer ${token}` } }),
    me: (token: string): Promise<CoreUser> =>
      request<CoreUser>('/v1/auth/me', { headers: { authorization: `Bearer ${token}` } }),
    authStatus: (): Promise<{ has_users: boolean }> =>
      request<{ has_users: boolean }>('/v1/auth/status'),
    listUsers: (): Promise<CoreUserRow[]> =>
      request<CoreUserRow[]>('/v1/admin/users'),
    createUser: (user: { username: string; password: string; role: string }): Promise<CoreUser> =>
      request<CoreUser>('/v1/admin/users', { method: 'POST', body: user }),
    updateUser: (id: string, role: string): Promise<CoreUser> =>
      request<CoreUser>(`/v1/admin/users/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: { role }
      }),
    deleteUser: (id: string): Promise<void> =>
      request<void>(`/v1/admin/users/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    resetUserPassword: (id: string, password: string): Promise<void> =>
      request<void>(`/v1/admin/users/${encodeURIComponent(id)}/reset-password`, {
        method: 'POST',
        body: { password }
      }),
    listEntities: (): Promise<CoreEntity[]> => request<CoreEntity[]>('/v1/meta/entities'),
    listEntitiesForUser: (): Promise<CoreEntity[]> => request<CoreEntity[]>('/v1/entities'),
    getEntity: (id: string): Promise<CoreEntityDetail> =>
      request<CoreEntityDetail>(`/v1/meta/entities/${encodeURIComponent(id)}`),
    createEntity: (entity: CoreEntity): Promise<CoreEntity> =>
      request<CoreEntity>('/v1/meta/entities', {
        method: 'POST',
        body: entity
      }),
    updateEntity: (id: string, entity: { name: string; label: string; module?: string | null }): Promise<CoreEntity> =>
      request<CoreEntity>(`/v1/meta/entities/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: entity
      }),
    getEntityOptions: (id: string): Promise<CoreEntityOption[]> =>
      request<CoreEntityOption[]>(`/v1/entities/${encodeURIComponent(id)}/options`),
    deleteEntity: (id: string): Promise<void> =>
      request<void>(`/v1/meta/entities/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      }),
    createField: (entityId: string, field: { name: string; type: string; required: boolean; is_status: boolean; ref_entity?: string | null; computed_expr?: string | null }): Promise<CoreField> =>
      request<CoreField>(`/v1/meta/entities/${encodeURIComponent(entityId)}/fields`, {
        method: 'POST',
        body: field
      }),
    updateField: (id: string, field: { name: string; type: string; required: boolean; is_status: boolean; ref_entity?: string | null; computed_expr?: string | null }): Promise<CoreField> =>
      request<CoreField>(`/v1/meta/fields/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: field
      }),
    deleteField: (id: string): Promise<void> =>
      request<void>(`/v1/meta/fields/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      }),
    createFieldOption: (fieldId: string, option: { value: string; label: string }): Promise<CoreFieldOption> =>
      request<CoreFieldOption>(`/v1/meta/fields/${encodeURIComponent(fieldId)}/options`, {
        method: 'POST',
        body: option
      }),
    updateFieldOption: (id: string, option: { value: string; label: string }): Promise<CoreFieldOption> =>
      request<CoreFieldOption>(`/v1/meta/options/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: option
      }),
    deleteFieldOption: (id: string): Promise<void> =>
      request<void>(`/v1/meta/options/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      }),
    listDocuments: (
      entityId: string,
      limit = 50,
      offset = 0,
      options: { search?: string; status?: string; sortBy?: string; sortDir?: string; viewId?: string } = {}
    ): Promise<CoreDocumentList> => {
      const { viewId, ...rest } = options
      return request<CoreDocumentList>('/v1/documents', {
        query: { entity_id: entityId, limit, offset, ...rest, ...(viewId ? { view_id: viewId } : {}) }
      })
    },
    createDocument: (input: CreateDocumentInput): Promise<CoreDocument> =>
      request<CoreDocument>('/v1/documents', {
        method: 'POST',
        body: input
      }),
    getDocument: (id: string): Promise<CoreDocument> =>
      request<CoreDocument>(`/v1/documents/${encodeURIComponent(id)}`),
    updateDocument: (
      id: string,
      payload: Record<string, unknown>,
      expectedUpdatedAt?: string
    ): Promise<CoreDocument> =>
      request<CoreDocument>(`/v1/documents/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: { payload, ...(expectedUpdatedAt ? { expected_updated_at: expectedUpdatedAt } : {}) }
      }),
    deleteDocument: async (id: string): Promise<void> => {
      await request<void>(`/v1/documents/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      })
    },
    listDocumentAudit: (
      id: string,
      limit = 50,
      offset = 0
    ): Promise<CoreAuditList> =>
      request<CoreAuditList>(`/v1/documents/${encodeURIComponent(id)}/audit`, {
        query: { limit, offset }
      }),
    listGlobalAudit: (
      limit = 50,
      offset = 0,
      options: { entityId?: string; action?: string; search?: string } = {}
    ): Promise<CoreGlobalAuditList> =>
      request<CoreGlobalAuditList>('/v1/audit', {
        query: {
          limit,
          offset,
          entity_id: options.entityId,
          action: options.action,
          search: options.search
        }
      }),
    getWorkflow: (entityId: string): Promise<CoreWorkflowDefinition> =>
      request<CoreWorkflowDefinition>(`/v1/meta/entities/${encodeURIComponent(entityId)}/workflow`),
    createWorkflowState: (entityId: string, state: { name: string; label: string }): Promise<CoreWorkflowState> =>
      request<CoreWorkflowState>(`/v1/meta/entities/${encodeURIComponent(entityId)}/workflow/states`, {
        method: 'POST',
        body: state
      }),
    updateWorkflowState: (id: string, state: { label: string }): Promise<CoreWorkflowState> =>
      request<CoreWorkflowState>(`/v1/meta/workflow/states/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: state
      }),
    deleteWorkflowState: (id: string): Promise<void> =>
      request<void>(`/v1/meta/workflow/states/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      }),
    createWorkflowTransition: (entityId: string, transition: { from_state: string; to_state: string; action: string }): Promise<CoreWorkflowTransition> =>
      request<CoreWorkflowTransition>(`/v1/meta/entities/${encodeURIComponent(entityId)}/workflow/transitions`, {
        method: 'POST',
        body: transition
      }),
    getEntityPermissions: (entityId: string): Promise<CoreEntityPermission[]> =>
      request<CoreEntityPermission[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/permissions`),
    updateEntityPermissions: (
      entityId: string,
      permissions: { role: string; can_view: boolean; can_edit: boolean }[]
    ): Promise<CoreEntityPermission[]> =>
      request<CoreEntityPermission[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/permissions`, {
        method: 'PUT',
        body: { permissions }
      }),
    getFieldPermissions: (entityId: string): Promise<CoreFieldPermission[]> =>
      request<CoreFieldPermission[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/field-permissions`),
    updateFieldPermissions: (
      entityId: string,
      permissions: { field_id: string; role: string; can_view: boolean; can_edit: boolean }[]
    ): Promise<CoreFieldPermission[]> =>
      request<CoreFieldPermission[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/field-permissions`, {
        method: 'PUT',
        body: { permissions }
      }),
    listEntityViews: (entityId: string): Promise<CoreEntityView[]> =>
      request<CoreEntityView[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/views`),
    createEntityView: (
      entityId: string,
      view: { name: string; config: Record<string, unknown> }
    ): Promise<CoreEntityView> =>
      request<CoreEntityView>(`/v1/meta/entities/${encodeURIComponent(entityId)}/views`, {
        method: 'POST',
        body: view
      }),
    getEntityView: (id: string): Promise<CoreEntityView> =>
      request<CoreEntityView>(`/v1/meta/views/${encodeURIComponent(id)}`),
    deleteEntityView: (id: string): Promise<void> =>
      request<void>(`/v1/meta/views/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    getFormLayout: (entityId: string): Promise<CoreFormLayout> =>
      request<CoreFormLayout>(`/v1/meta/entities/${encodeURIComponent(entityId)}/form-layout`),
    updateFormLayout: (entityId: string, config: Record<string, unknown>): Promise<CoreFormLayout> =>
      request<CoreFormLayout>(`/v1/meta/entities/${encodeURIComponent(entityId)}/form-layout`, {
        method: 'PUT',
        body: { config }
      }),
    getFormLayoutForUser: (entityId: string): Promise<CoreFormLayout> =>
      request<CoreFormLayout>(`/v1/entities/${encodeURIComponent(entityId)}/form-layout`),
    listNotificationRules: (entityId: string): Promise<CoreNotificationRule[]> =>
      request<CoreNotificationRule[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/notification-rules`),
    createNotificationRule: (
      entityId: string,
      rule: { trigger?: string; target_url: string; active?: boolean }
    ): Promise<CoreNotificationRule> =>
      request<CoreNotificationRule>(`/v1/meta/entities/${encodeURIComponent(entityId)}/notification-rules`, {
        method: 'POST',
        body: rule
      }),
    updateNotificationRule: (
      id: string,
      rule: { trigger?: string; target_url?: string; active?: boolean }
    ): Promise<CoreNotificationRule> =>
      request<CoreNotificationRule>(`/v1/meta/notification-rules/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: rule
      }),
    deleteNotificationRule: (id: string): Promise<void> =>
      request<void>(`/v1/meta/notification-rules/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    listNotificationDeliveries: (limit = 50, offset = 0): Promise<CoreNotificationDeliveryList> =>
      request<CoreNotificationDeliveryList>('/v1/admin/notification-deliveries', {
        query: { limit, offset }
      }),
    getReportAggregate: (entityId: string, groupBy: string): Promise<CoreReportBucket[]> =>
      request<CoreReportBucket[]>('/v1/reports/aggregate', {
        query: { entity_id: entityId, group_by: groupBy }
      }),
    listReports: (entityId: string): Promise<CoreReport[]> =>
      request<CoreReport[]>(`/v1/meta/entities/${encodeURIComponent(entityId)}/reports`),
    createReport: (
      entityId: string,
      report: { name: string; config: Record<string, unknown> }
    ): Promise<CoreReport> =>
      request<CoreReport>(`/v1/meta/entities/${encodeURIComponent(entityId)}/reports`, {
        method: 'POST',
        body: report
      }),
    getReport: (id: string): Promise<CoreReport> =>
      request<CoreReport>(`/v1/meta/reports/${encodeURIComponent(id)}`),
    deleteReport: (id: string): Promise<void> =>
      request<void>(`/v1/meta/reports/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    listReportsForUser: (entityId: string): Promise<CoreReport[]> =>
      request<CoreReport[]>(`/v1/entities/${encodeURIComponent(entityId)}/reports`),
    getReportForUser: (id: string): Promise<CoreReport> =>
      request<CoreReport>(`/v1/reports/${encodeURIComponent(id)}`),
    deleteWorkflowTransition: (id: string): Promise<void> =>
      request<void>(`/v1/meta/workflow/transitions/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      }),
    exportDocuments: (entityId: string): Promise<string> =>
      request<string>(`/v1/meta/entities/${encodeURIComponent(entityId)}/export`),
    previewImport: (entityId: string, csv: string) =>
      request(`/v1/meta/entities/${encodeURIComponent(entityId)}/import/preview`, { method: 'POST', body: csv, headers: { 'content-type': 'text/csv' } }),
    confirmImport: (entityId: string, csv: string) =>
      request(`/v1/meta/entities/${encodeURIComponent(entityId)}/import/confirm`, { method: 'POST', body: csv, headers: { 'content-type': 'text/csv' } }),
    transitionDocument: (id: string, action: string, expectedUpdatedAt?: string): Promise<CoreDocument> =>
      request<CoreDocument>(`/v1/documents/${encodeURIComponent(id)}/transition`, {
        method: 'POST',
        body: { action, ...(expectedUpdatedAt ? { expected_updated_at: expectedUpdatedAt } : {}) }
      }),
    getEntityForUser: (id: string): Promise<CoreEntityWithPermission> =>
      request<CoreEntityWithPermission>(`/v1/entities/${encodeURIComponent(id)}`),
    getWorkflowForUser: (entityId: string): Promise<CoreWorkflowDefinition> =>
      request<CoreWorkflowDefinition>(`/v1/entities/${encodeURIComponent(entityId)}/workflow`),
    listEntityViewsForUser: (entityId: string): Promise<CoreEntityView[]> =>
      request<CoreEntityView[]>(`/v1/entities/${encodeURIComponent(entityId)}/views`),
    getEntityViewForUser: (id: string): Promise<CoreEntityView> =>
      request<CoreEntityView>(`/v1/views/${encodeURIComponent(id)}`),
    exportDocumentsForUser: (entityId: string): Promise<string> =>
      request<string>(`/v1/entities/${encodeURIComponent(entityId)}/export`),
    previewImportForUser: (entityId: string, csv: string) =>
      request(`/v1/entities/${encodeURIComponent(entityId)}/import/preview`, { method: 'POST', body: csv, headers: { 'content-type': 'text/csv' } }),
    confirmImportForUser: (entityId: string, csv: string) =>
      request(`/v1/entities/${encodeURIComponent(entityId)}/import/confirm`, { method: 'POST', body: csv, headers: { 'content-type': 'text/csv' } }),
    exportWorkbook: (): Promise<ArrayBuffer> =>
      request<ArrayBuffer>('/v1/entities/export', { responseType: 'arrayBuffer' }),
    previewWorkbookImport: (bytes: ArrayBuffer | Uint8Array): Promise<CoreMultiImportPreview> =>
      request<CoreMultiImportPreview>('/v1/entities/import/preview', {
        method: 'POST',
        body: bytes as BodyInit,
        headers: { 'content-type': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' }
      }),
    confirmWorkbookImport: (bytes: ArrayBuffer | Uint8Array): Promise<CoreMultiImportResult> =>
      request<CoreMultiImportResult>('/v1/entities/import/confirm', {
        method: 'POST',
        body: bytes as BodyInit,
        headers: { 'content-type': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' }
      }),
    getDashboardCounts: (entityId: string): Promise<CoreStatusCount[]> =>
      request<CoreStatusCount[]>('/v1/dashboard/counts', { query: { entity_id: entityId } }),
    getPmSummary: (entityId: string): Promise<CorePmSummary> =>
      request<CorePmSummary>('/v1/dashboard/pm', { query: { entity_id: entityId } }),
    getAdminStatus: (): Promise<CoreAdminStatus> =>
      request<CoreAdminStatus>('/v1/admin/status'),
    createBackup: (): Promise<{ path: string }> =>
      request<{ path: string }>('/v1/admin/backup', { method: 'POST' }),
    listBackups: (): Promise<{ items: CoreBackupInfo[] }> =>
      request<{ items: CoreBackupInfo[] }>('/v1/admin/backups'),
    downloadBackup: (name: string): Promise<Blob> =>
      request<Blob>(`/v1/admin/backups/${encodeURIComponent(name)}`),
    restoreBackup: (path: string): Promise<{ message: string; staged: string }> =>
      request<{ message: string; staged: string }>('/v1/admin/restore', {
        method: 'POST',
        body: { path, force: true }
      }),
    restartCore: (): Promise<{ message: string }> =>
      request<{ message: string }>('/v1/admin/restart', { method: 'POST' })
  }
}
