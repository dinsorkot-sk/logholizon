export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const module = getRouterParam(event, 'module')!
  const entity = getRouterParam(event, 'entity')!
  return await $fetch(`${config.coreUrl}/v1/modules/${module}/entities/${entity}`, {
    method: 'POST',
    body: await readBody(event),
    headers: { authorization: getRequestHeader(event, 'authorization') || '' },
  })
})
