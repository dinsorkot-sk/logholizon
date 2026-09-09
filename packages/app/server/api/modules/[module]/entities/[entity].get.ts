export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const module = getRouterParam(event, 'module')!
  const entity = getRouterParam(event, 'entity')!
  return await $fetch(`${config.coreUrl}/v1/modules/${module}/entities/${entity}`, {
    query: getQuery(event),
    headers: { authorization: getRequestHeader(event, 'authorization') || '' },
  })
})
