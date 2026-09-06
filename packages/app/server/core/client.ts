export type CoreEntity = { id: string; name: string; label: string }

export type CoreUser = { id: string; username: string; role: string }
export type CoreSession = { token: string; user: CoreUser }

export type CoreFieldOption = { id: string; value: string; label: string }

export type CoreField = {
  id: string
  name: string
  type: string
  required: boolean
  is_status: boolean
  position: number
  options: CoreFieldOption[]
}

export type CoreEntityDetail = CoreEntity & { fields: CoreField[] }

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
}

export type CoreAuditList = {
  items: CoreAuditEntry[]
  total: number
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

type CoreError = { code?: string; message?: string }

export function coreClient() {
  const config = useRuntimeConfig()
  const baseURL = config.coreUrl.replace(/\/$/, '')

  async function request<T>(path: string, options?: Parameters<typeof $fetch<T>>[1]): Promise<T> {
    try {
      return (await $fetch<T>(`${baseURL}${path}`, options)) as T
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
    logout: (token: string): Promise<void> =>
      request<void>('/v1/auth/logout', { method: 'POST', headers: { authorization: `Bearer ${token}` } }),
    me: (token: string): Promise<CoreUser> =>
      request<CoreUser>('/v1/auth/me', { headers: { authorization: `Bearer ${token}` } }),
    listEntities: (): Promise<CoreEntity[]> => request<CoreEntity[]>('/v1/meta/entities'),
    getEntity: (id: string): Promise<CoreEntityDetail> =>
      request<CoreEntityDetail>(`/v1/meta/entities/${encodeURIComponent(id)}`),
    createEntity: (entity: CoreEntity): Promise<CoreEntity> =>
      request<CoreEntity>('/v1/meta/entities', {
        method: 'POST',
        body: entity
      }),
    updateEntity: (id: string, entity: { name: string; label: string }): Promise<CoreEntity> =>
      request<CoreEntity>(`/v1/meta/entities/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: entity
      }),
    deleteEntity: (id: string): Promise<void> =>
      request<void>(`/v1/meta/entities/${encodeURIComponent(id)}`, {
        method: 'DELETE'
      }),
    createField: (entityId: string, field: { name: string; type: string; required: boolean; is_status: boolean }): Promise<CoreField> =>
      request<CoreField>(`/v1/meta/entities/${encodeURIComponent(entityId)}/fields`, {
        method: 'POST',
        body: field
      }),
    updateField: (id: string, field: { name: string; type: string; required: boolean; is_status: boolean }): Promise<CoreField> =>
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
      options: { search?: string; status?: string; sortBy?: string; sortDir?: string } = {}
    ): Promise<CoreDocumentList> =>
      request<CoreDocumentList>('/v1/documents', {
        query: { entity_id: entityId, limit, offset, ...options }
      }),
    createDocument: (input: CreateDocumentInput): Promise<CoreDocument> =>
      request<CoreDocument>('/v1/documents', {
        method: 'POST',
        body: input
      }),
    getDocument: (id: string): Promise<CoreDocument> =>
      request<CoreDocument>(`/v1/documents/${encodeURIComponent(id)}`),
    updateDocument: (
      id: string,
      payload: Record<string, unknown>
    ): Promise<CoreDocument> =>
      request<CoreDocument>(`/v1/documents/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: { payload }
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
    transitionDocument: (id: string, action: string): Promise<CoreDocument> =>
      request<CoreDocument>(`/v1/documents/${encodeURIComponent(id)}/transition`, {
        method: 'POST',
        body: { action }
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
