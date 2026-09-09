export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  return proxyCore(event, `/v1/modules/${encodeURIComponent(id || '')}/review`, { method: 'POST' })
})
