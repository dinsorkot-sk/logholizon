export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const id = getRouterParam(event, 'id')
  const relationId = getRouterParam(event, 'relation_id')
  return await $fetch(`${config.coreApiBase}/v1/entities/${encodeURIComponent(id!)}/relations/${encodeURIComponent(relationId!)}`, {
    headers: { authorization: getHeader(event, 'authorization') || '' }
  })
})
