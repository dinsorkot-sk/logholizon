export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const id = getRouterParam(event, 'id')
  return await $fetch(`${config.coreApiBase}/v1/meta/entities/${encodeURIComponent(id!)}/relations`, {
    method: 'POST',
    body: await readBody(event),
    headers: { authorization: getHeader(event, 'authorization') || '' }
  })
})
