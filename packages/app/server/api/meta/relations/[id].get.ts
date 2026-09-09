export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const id = getRouterParam(event, 'id')
  return await $fetch(`${config.coreApiBase}/v1/meta/relations/${encodeURIComponent(id!)}`, {
    headers: { authorization: getHeader(event, 'authorization') || '' }
  })
})
