type CoreEntity = { id: string; name: string; label: string }

type CoreError = { code?: string; message?: string }

export function coreClient() {
  const config = useRuntimeConfig()
  const baseURL = config.coreUrl.replace(/\/$/, '')

  async function request<T>(path: string, options?: Parameters<typeof $fetch<T>>[1]): Promise<T> {
    try {
      return await $fetch<T>(`${baseURL}${path}`, options) as T
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
    createEntity: (entity: CoreEntity): Promise<CoreEntity> => request<CoreEntity>('/v1/meta/entities', {
      method: 'POST',
      body: entity
    })
  }
}
