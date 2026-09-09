export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const id = getRouterParam(event, 'id')
  return await $fetch(`${config.coreApiBase}/v1/meta/relations/${encodeURIComponent(id!)}`, {
    method: 'DELETE', headers: { authorization: getHeader(event, 'authorization') || '' }
  })
})
