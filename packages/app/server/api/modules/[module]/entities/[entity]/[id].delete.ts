export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const module = getRouterParam(event, 'module')!
  const entity = getRouterParam(event, 'entity')!
  const id = getRouterParam(event, 'id')!
  return await $fetch(`${config.coreUrl}/v1/modules/${module}/entities/${entity}/${id}`, {
    method: 'DELETE',
    headers: { authorization: getRequestHeader(event, 'authorization') || '' },
  })
})
