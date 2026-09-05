export type CoreEntity = { id: string; name: string; label: string }

export type CoreFieldOption = { id: string; value: string; label: string }

export type CoreField = {
  id: string
  name: string
  type: string
  required: boolean
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
    listEntities: (): Promise<CoreEntity[]> => request<CoreEntity[]>('/v1/meta/entities'),
    getEntity: (id: string): Promise<CoreEntityDetail> =>
      request<CoreEntityDetail>(`/v1/meta/entities/${encodeURIComponent(id)}`),
    createEntity: (entity: CoreEntity): Promise<CoreEntity> =>
      request<CoreEntity>('/v1/meta/entities', {
        method: 'POST',
        body: entity
      }),
    listDocuments: (
      entityId: string,
      limit = 50,
      offset = 0
    ): Promise<CoreDocumentList> =>
      request<CoreDocumentList>('/v1/documents', {
        query: { entity_id: entityId, limit, offset }
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
      })
  }
}
